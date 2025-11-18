use ethers_core::{
    types::{
        transaction::eip1559::Eip1559TransactionRequest, Address as EthAddress,
        NameOrAddress, Signature as EthSignature, TransactionRequest, U256, U64,
    },
    utils::{rlp::Encodable, to_checksum},
};
use ic_cdk::api::management_canister::ecdsa::{
    EcdsaCurve, EcdsaKeyId, EcdsaPublicKeyArgument, EcdsaPublicKeyResponse,
    SignWithEcdsaArgument, SignWithEcdsaResponse,
};
use ic_cdk::api::management_canister::main::CanisterId;
use k256::{
    ecdsa::{RecoveryId, Signature as K256Signature, VerifyingKey},
    elliptic_curve::{consts::U32, generic_array},
    FieldBytes,
};
use sha3::{Digest, Keccak256};
use std::str::FromStr;

const DEFAULT_DERIVATION_PATH: Vec<Vec<u8>> = vec![];

// Ethereum Sepolia testnet
const CHAIN_ID: u64 = 11155111;

// ============================================================================
// ECDSA KEY MANAGEMENT
// ============================================================================

fn get_ecdsa_key_id() -> EcdsaKeyId {
    EcdsaKeyId {
        curve: EcdsaCurve::Secp256k1,
        name: "dfx_test_key".to_string(), // Use test key for local development
    }
}

async fn get_ecdsa_public_key() -> Result<Vec<u8>, String> {
    let request = EcdsaPublicKeyArgument {
        canister_id: None,
        derivation_path: DEFAULT_DERIVATION_PATH.clone(),
        key_id: get_ecdsa_key_id(),
    };

    let (response,): (EcdsaPublicKeyResponse,) = ic_cdk::call(
        CanisterId::management_canister(),
        "ecdsa_public_key",
        (request,),
    )
    .await
    .map_err(|(code, msg)| format!("Failed to get ECDSA public key: {} ({:?})", msg, code))?;

    Ok(response.public_key)
}

// ============================================================================
// ADDRESS GENERATION
// ============================================================================

/// Get our canister's Ethereum address
#[ic_cdk_macros::update]
pub async fn get_eth_address() -> Result<String, String> {
    ic_cdk::println!("Generating Ethereum address...");

    let pub_key = get_ecdsa_public_key().await?;

    ic_cdk::println!("Got public key: {} bytes", pub_key.len());

    // Convert public key to Ethereum address
    let key = ethers_core::k256::ecdsa::VerifyingKey::from_sec1_bytes(&pub_key)
        .map_err(|e| format!("Failed to parse key: {}", e))?;

    let address = ethers_core::utils::public_key_to_address(&key);

    let checksummed = to_checksum(&address, None);

    ic_cdk::println!("Generated address: {}", checksummed);

    Ok(checksummed)
}

// ============================================================================
// TRANSACTION SIGNING
// ============================================================================

/// Calculate the recovery ID (v value) for an Ethereum signature
fn calculate_recovery_id(
    hash: &[u8],
    signature: &[u8],
    pub_key: &[u8],
    chain_id: u64,
) -> Result<U64, String> {
    // Parse R and S from signature
    let r_bytes: FieldBytes = *generic_array::GenericArray::from_slice(&signature[0..32]);
    let s_bytes: FieldBytes = *generic_array::GenericArray::from_slice(&signature[32..64]);

    let k256_sig = K256Signature::from_scalars(r_bytes, s_bytes)
        .map_err(|e| format!("Failed to parse k256 signature: {}", e))?;

    let verifying_key = VerifyingKey::from_sec1_bytes(pub_key)
        .map_err(|e| format!("Failed to parse verifying key: {}", e))?;

    // Try both recovery IDs
    for recovery_id in [0u8, 1u8] {
        let rec_id = RecoveryId::try_from(recovery_id).unwrap();
        if let Ok(recovered_key) = VerifyingKey::recover_from_prehash(hash, &k256_sig, rec_id) {
            if recovered_key == verifying_key {
                // EIP-155: v = {0,1} + 35 + (chainId * 2)
                return Ok(U64::from(recovery_id as u64 + 35 + (chain_id * 2)));
            }
        }
    }

    Err("Could not calculate recovery ID".to_string())
}

/// Sign an Ethereum transaction using threshold ECDSA
pub async fn sign_eth_transaction(
    to: String,
    value: u64,
    data: Vec<u8>,
    nonce: u64,
    max_fee_per_gas: u64,
    max_priority_fee_per_gas: u64,
) -> Result<String, String> {
    ic_cdk::println!("Signing Ethereum transaction...");

    // Get public key
    let pub_key = get_ecdsa_public_key().await?;

    // Parse destination address
    let to_address =
        EthAddress::from_str(&to).map_err(|e| format!("Invalid 'to' address: {}", e))?;

    // Build EIP-1559 transaction
    let tx = Eip1559TransactionRequest {
        to: Some(NameOrAddress::Address(to_address)),
        from: None,
        nonce: Some(U256::from(nonce)),
        value: Some(U256::from(value)),
        data: Some(data.into()),
        gas: Some(U256::from(21000u64)), // Simple transfer gas limit
        max_fee_per_gas: Some(U256::from(max_fee_per_gas)),
        max_priority_fee_per_gas: Some(U256::from(max_priority_fee_per_gas)),
        chain_id: Some(U64::from(CHAIN_ID)),
        access_list: Default::default(),
    };

    // RLP encode the transaction
    let rlp_bytes = tx.rlp();
    let mut rlp_encoded = vec![0x02]; // EIP-1559 transaction type
    rlp_encoded.extend_from_slice(&rlp_bytes);

    // Hash the encoded transaction
    let tx_hash = Keccak256::digest(&rlp_encoded).to_vec();

    ic_cdk::println!("Transaction hash: {}", hex::encode(&tx_hash));

    // Sign with threshold ECDSA
    let request = SignWithEcdsaArgument {
        message_hash: tx_hash.clone(),
        derivation_path: DEFAULT_DERIVATION_PATH.clone(),
        key_id: get_ecdsa_key_id(),
    };

    let cycles_needed = 26_153_846_153u128;

    let (response,): (SignWithEcdsaResponse,) =
        ic_cdk::api::call::call_with_payment128(
            CanisterId::management_canister(),
            "sign_with_ecdsa",
            (request,),
            cycles_needed,
        )
        .await
        .map_err(|(code, msg)| format!("Failed to sign: {} ({:?})", msg, code))?;

    let signature = response.signature;

    ic_cdk::println!("Got signature: {} bytes", signature.len());

    // Extract r, s, v
    let r = U256::from_big_endian(&signature[0..32]);
    let s = U256::from_big_endian(&signature[32..64]);
    let v = calculate_recovery_id(&tx_hash, &signature, &pub_key, CHAIN_ID)?;

    let eth_signature = EthSignature {
        r,
        s,
        v: v.as_u64(),
    };

    // RLP encode signed transaction
    let signed_rlp = tx.rlp_signed(&eth_signature);
    let mut signed_encoded = vec![0x02];
    signed_encoded.extend_from_slice(&signed_rlp);

    let raw_tx_hex = format!("0x{}", hex::encode(signed_encoded));

    ic_cdk::println!("Signed transaction ready");

    Ok(raw_tx_hex)
}

// ============================================================================
// UNISWAP INTEGRATION
// ============================================================================

/// Get a quote from Uniswap V3 (via HTTPS outcalls)
pub async fn get_uniswap_quote(
    token_in: String,
    token_out: String,
    amount_in: u64,
) -> Result<u64, String> {
    ic_cdk::println!(
        "Getting Uniswap quote: {} {} for {}",
        amount_in,
        token_in,
        token_out
    );

    // In production, this would make an HTTPS outcall to:
    // - Uniswap API or quoter contract
    // - Return expected amount out

    // For demo, return a mock quote (98% of input, simulating 2% slippage)
    let amount_out = (amount_in as f64 * 0.98) as u64;

    ic_cdk::println!("Quote: {} {} -> {} {}", amount_in, token_in, amount_out, token_out);

    Ok(amount_out)
}

/// Build Uniswap V3 swap calldata
pub fn build_uniswap_swap_calldata(
    token_in: EthAddress,
    token_out: EthAddress,
    amount_in: U256,
    amount_out_minimum: U256,
    recipient: EthAddress,
) -> Vec<u8> {
    // Uniswap V3 Router exactInputSingle function selector
    let function_selector = hex::decode("414bf389").unwrap(); // exactInputSingle

    // Build parameters struct (simplified)
    // In production, you'd use ethers-rs ABI encoding
    let mut calldata = function_selector;

    // For demo, we'll return a placeholder
    // Real implementation would properly ABI-encode:
    // - tokenIn
    // - tokenOut
    // - fee (pool fee tier)
    // - recipient
    // - deadline
    // - amountIn
    // - amountOutMinimum
    // - sqrtPriceLimitX96

    ic_cdk::println!("Built Uniswap swap calldata: {} bytes", calldata.len());

    calldata
}

// ============================================================================
// HIGH-LEVEL SETTLEMENT FUNCTION
// ============================================================================

/// Execute Ethereum settlement via Uniswap
/// net_position: positive = need to buy ETH, negative = need to sell ETH
#[ic_cdk_macros::update]
pub async fn execute_eth_settlement(
    net_position: i64,
) -> Result<String, String> {
    ic_cdk::println!("Executing Ethereum settlement: {} wei", net_position);

    if net_position == 0 {
        return Ok("Net position is balanced - no settlement needed".to_string());
    }

    // Get our address
    let our_address = get_eth_address().await?;

    // For demo purposes, we'll build a simple ETH transfer
    // In production, this would:
    // 1. Get quote from Uniswap
    // 2. Build swap transaction
    // 3. Sign and broadcast

    // Get current nonce (would need to query via HTTPS outcall)
    let nonce = 0u64; // Placeholder

    // Get gas prices (would need to query via HTTPS outcall)
    let max_fee = 50_000_000_000u64; // 50 gwei
    let max_priority_fee = 2_000_000_000u64; // 2 gwei

    // For demo, send to a test address
    let recipient = "0x0000000000000000000000000000000000000000"; // Burn address for demo

    // Sign transaction
    let signed_tx = sign_eth_transaction(
        recipient.to_string(),
        net_position.abs() as u64,
        vec![], // No calldata for simple transfer
        nonce,
        max_fee,
        max_priority_fee,
    )
    .await?;

    ic_cdk::println!("Transaction signed. Ready to broadcast.");

    // In production, broadcast via HTTPS outcall to:
    // - Infura
    // - Alchemy
    // - Or direct node

    Ok(format!(
        "Ethereum settlement transaction built: {}... (not broadcasted in demo)",
        &signed_tx[..20]
    ))
}

// ============================================================================
// TESTING/DEMO FUNCTIONS
// ============================================================================

/// Send a test Ethereum transaction
#[ic_cdk_macros::update]
pub async fn send_test_eth(
    to_address: String,
    amount_wei: u64,
) -> Result<String, String> {
    ic_cdk::println!("Sending test ETH: {} wei to {}", amount_wei, to_address);

    let signed_tx = sign_eth_transaction(
        to_address,
        amount_wei,
        vec![],
        0, // nonce
        50_000_000_000,  // max fee
        2_000_000_000,   // max priority fee
    )
    .await?;

    Ok(format!("Test transaction signed: {}...", &signed_tx[..20]))
}

/// Build a Uniswap swap transaction (for demo)
#[ic_cdk_macros::update]
pub async fn build_uniswap_swap(
    amount_in: u64,
    slippage_bps: u64, // basis points (e.g., 100 = 1%)
) -> Result<String, String> {
    ic_cdk::println!("Building Uniswap swap: {} wei with {}bps slippage", amount_in, slippage_bps);

    // WETH and USDC addresses on Sepolia (example)
    let weth = "0x7b79995e5f793A07Bc00c21412e50Ecae098E7f9";
    let usdc = "0x94a9D9AC8a22534E3FaCa9F4e7F2E2cf85d5E4C8";

    // Get quote
    let quote = get_uniswap_quote(
        weth.to_string(),
        usdc.to_string(),
        amount_in,
    ).await?;

    // Calculate minimum output with slippage
    let min_output = quote - (quote * slippage_bps / 10000);

    ic_cdk::println!("Quote: {} wei, Min output: {} wei", quote, min_output);

    Ok(format!(
        "Uniswap swap: {} {} for at least {} {}",
        amount_in, weth, min_output, usdc
    ))
}