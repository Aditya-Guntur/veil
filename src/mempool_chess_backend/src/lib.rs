// --- 1. REPLACE ALL 'use' STATEMENTS WITH THIS BLOCK ---
use candid::{CandidType, Decode, Encode, Principal};
use ic_cdk::api::time; // From original file
use ic_cdk_macros::*; // From original file
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory}; // From original file
use ic_stable_structures::{storable::Bound, DefaultMemoryImpl, StableBTreeMap, Storable}; // From original file
use serde::Deserialize; // From original file
use std::borrow::Cow; // From original file
use std::cell::RefCell; // From original file
use std::str::FromStr; // For EthAddress::from_str
use ic_cdk::api::id; // To check our own canister ID
use ic_cdk::api::management_canister::bitcoin::GetBalanceRequest; // We'll need this too
// --- Your Project ---
mod auction;

// --- ADD THESE TWO LINES BACK ---
type OrderId = u64;
type Timestamp = u64; // Nanoseconds

// --- IC Management Canister (NEW PATHS - NOT DEPRECATED) ---
// --- IC Management Canister (Corrected Paths) ---
use ic_cdk::{
    call::RejectCode,
    api::management_canister::{ // <--- ADDS 'api::'
    bitcoin::{
    BitcoinNetwork, GetUtxosRequest, GetUtxosResponse,
    },
    ecdsa::{
    EcdsaCurve, EcdsaKeyId, EcdsaPublicKeyArgument, EcdsaPublicKeyResponse,
    SignWithEcdsaArgument, SignWithEcdsaResponse,
    },
    schnorr::{
    SchnorrAlgorithm, SchnorrKeyId, SchnorrPublicKeyArgument, SchnorrPublicKeyResponse,
    },
    },  
};
use ic_cdk::management_canister::CanisterId; // <-- ADDS THE CORRECT CanisterId PATH

// --- Ethers (Ethereum) ---
// --- Ethers (Ethereum) ---
// --- Ethers (Ethereum) ---
use ethers_core::{
    types::{
        transaction::eip1559::Eip1559TransactionRequest, Address as EthAddress, NameOrAddress,
        Signature as EthSignature, U256, U64,
    },
    utils::{to_checksum, rlp::{Encodable, RlpStream}},
};
// --- K256 (Crypto) ---
use k256::{
    ecdsa::{RecoveryId, Signature as K256Signature, VerifyingKey},
    elliptic_curve::{consts::U32, generic_array},
    FieldBytes,
};
use sha3::{Digest, Keccak256};

// --- Bitcoin ---
use bitcoin::{
    network::Network as BtcNetwork,
    secp256k1::{All, Secp256k1}, // <-- This adds 'All'
    Address as BtcAddress,      // <-- This adds 'BtcAddress'
    PublicKey as BtcPublicKey,
    XOnlyPublicKey,
};
// --- ADD THIS HELPER CODE BLOCK ---

// Derivation path for our canister's default key
const DEFAULT_DERIVATION_PATH: Vec<Vec<u8>> = vec![];

// The key ID for ECDSA (used for Ethereum)
fn get_ecdsa_key_id() -> EcdsaKeyId {
    EcdsaKeyId {
        curve: EcdsaCurve::Secp256k1,
        name: "dfx_test_key".to_string(), // Use the default test key
    }
}

fn bitcoin_canister_id() -> CanisterId {
    // Check if we are running on the local replica
    if id().to_text() == "aaaaa-aa" {
        // Mainnet Bitcoin canister ID
        CanisterId::from_text("g4xu7-jiaaa-aaaan-aaaaq-cai").unwrap()
    } else {
        // Local replica Bitcoin canister ID
        CanisterId::from_text("mxzaz-ziaaa-aaaar-qaada-cai").unwrap()
    }
}

// The key ID for Schnorr (used for Bitcoin)
fn get_schnorr_key_id() -> SchnorrKeyId {
    SchnorrKeyId {
        algorithm: SchnorrAlgorithm::Bip340secp256k1,
        name: "dfx_test_key".to_string(), // Use the default test key
    }
}

/// Helper function to find the correct recovery ID (v) for an Ethereum signature
fn calculate_recovery_id(
    hash: &[u8],
    signature: &[u8],
    pub_key: &[u8],
    chain_id: u64,
) -> Result<U64, String> {
    
    // 1. Parse R and S values from the 64-byte signature
    // We add the '*' to dereference the slice into an owned value,
    // which fixes the compiler errors.
    let r_bytes: FieldBytes = *generic_array::GenericArray::from_slice(&signature[0..32]);
    let s_bytes: FieldBytes = *generic_array::GenericArray::from_slice(&signature[32..64]);

    // 2. Create a k256 signature object
    let k256_sig = K256Signature::from_scalars(r_bytes, s_bytes)
        .map_err(|e| format!("Failed to parse k256 signature: {}", e))?;
    
    // 3. Create a k256 verifying key object
    let verifying_key = VerifyingKey::from_sec1_bytes(pub_key)
        .map_err(|e| format!("Failed to parse verifying key: {}", e))?;

    // 4. Iterate (0 and 1) to find the correct recovery ID
    for recovery_id in [0, 1] {
        let rec_id = RecoveryId::try_from(recovery_id).unwrap();
        if let Ok(recovered_key) =
            VerifyingKey::recover_from_prehash(hash, &k256_sig, rec_id)
        {
            if recovered_key == verifying_key {
                // EIP-155: v = {0,1} + 35 + (chainId * 2)
                return Ok(U64::from(recovery_id as u64 + 35 + (chain_id * 2)));
            }
        }
    }
    Err("Could not calculate recovery ID".to_string())
}

/// Helper function to find the correct recovery ID (v) for an Ethereum signature


#[derive(CandidType, Deserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Asset {
    BTC,
    ETH,
    ICP,
}

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum OrderType {
    Buy,
    Sell,
}

#[derive(CandidType, Deserialize, Clone)]
pub struct Order {
    pub id: OrderId,
    pub owner: Principal,
    pub order_type: OrderType,
    pub asset: Asset,
    pub amount: u64,
    pub price_limit: u64,
    pub created_at: Timestamp,
    pub encrypted_payload: Vec<u8>,
}

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq)]
pub enum RoundState {
    Pending,
    Active,
    Revealing,
    Executing,
}

#[derive(CandidType, Deserialize, Clone)]
pub struct State {
    pub round_id: u64,
    pub round_state: RoundState,
    pub round_start_time: Timestamp,
    pub round_duration_ns: u64,
    pub next_order_id: OrderId,
}

impl Default for State {
    fn default() -> Self {
        State {
            round_id: 0,
            round_state: RoundState::Pending,
            round_start_time: 0,
            round_duration_ns: 60_000_000_000,
            next_order_id: 0,
        }
    }
}

// --- 2. STABLE MEMORY (Our Database) ---

impl Storable for Order {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(&self).unwrap())
    }
    
    fn into_bytes(self) -> Vec<u8> {
        Encode!(&self).unwrap()
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }

    const BOUND: Bound = Bound::Unbounded;
}

// Memory setup
type Memory = VirtualMemory<DefaultMemoryImpl>;
const ORDERS_MEMORY_ID: MemoryId = MemoryId::new(1);

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));

    static STATE: RefCell<State> = RefCell::new(State::default());

    pub static ORDERS: RefCell<StableBTreeMap<OrderId, Order, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(ORDERS_MEMORY_ID))
        )
    );
}

// --- 3. API ENDPOINTS (Our Functions) ---

#[update]
fn submit_order(
    order_type: OrderType,
    asset: Asset,
    amount: u64,
    price_limit: u64,
    encrypted_payload: Vec<u8>,
) -> Result<OrderId, String> {
    let current_state = STATE.with(|s| s.borrow().clone());
    
    if current_state.round_state != RoundState::Active {
        ic_cdk::println!("Warning: Round is not active, but proceeding for test.");
    }
    
    let order_id = STATE.with(|s| {
        let mut state = s.borrow_mut();
        let id = state.next_order_id;
        state.next_order_id += 1;
        id
    });

    let new_order = Order {
        id: order_id,
        owner: ic_cdk::caller(),
        order_type,
        asset,
        amount,
        price_limit,
        created_at: time(),
        encrypted_payload,
    };

    ORDERS.with(|orders| {
        orders.borrow_mut().insert(order_id, new_order);
    });

    ic_cdk::println!("Order submitted successfully: ID {}", order_id);
    Ok(order_id)
}


#[update]
fn admin_start_round() -> String {
    STATE.with(|s| {
        let mut state = s.borrow_mut();
        state.round_id += 1;
        state.round_state = RoundState::Active;
        state.round_start_time = time();
        
        ORDERS.with(|orders| {
            let mut orders_mut = orders.borrow_mut();
            let keys: Vec<OrderId> = orders_mut.keys().collect();
            for k in keys {
                orders_mut.remove(&k);
            }
        });
        
        state.next_order_id = 0;

        format!("Round {} started. Accepting orders.", state.round_id)
    })
}

#[query]
fn get_round_state() -> State {
    STATE.with(|s| s.borrow().clone())
}

#[query]
fn get_order_count() -> u64 {
    ORDERS.with(|orders| orders.borrow().len())
}

#[update]
fn admin_run_clearing() -> String {
    STATE.with(|s| {
        s.borrow_mut().round_state = RoundState::Revealing;
    });

    match auction::find_clearing_price() {
        Ok(result) => {
            STATE.with(|s| {
                s.borrow_mut().round_state = RoundState::Executing;
            });
            format!(
                "Clearing successful! Price: {}, Volume: {}",
                result.clearing_price, result.buy_volume
            )
        }
        Err(e) => {
            STATE.with(|s| {
                s.borrow_mut().round_state = RoundState::Pending;
            });
            format!("Clearing failed: {}", e)
        }
    }
}

// --- ADD THESE FOUR FUNCTIONS ---

#[update]
async fn get_eth_address() -> Result<String, String> {
    ic_cdk::println!("Requesting ETH (ECDSA) public key...");

    // 1. Request the public key from the management canister
    let (response,): (EcdsaPublicKeyResponse,) = ic_cdk::call(
        bitcoin_canister_id(),
        "ecdsa_public_key",
        (EcdsaPublicKeyArgument {
            canister_id: None,
            derivation_path: DEFAULT_DERIVATION_PATH.clone(),
            key_id: get_ecdsa_key_id(),
        },),
    )
    .await
    .map_err(|(code, msg)| format!("Failed to get ECDSA public key: {} ({:?})", msg, code))?;

    let pub_key = response.public_key;
    ic_cdk::println!("Got public key: {}", hex::encode(&pub_key));

    // 2. Convert the public key to an Ethereum address
    let key = ethers_core::k256::ecdsa::VerifyingKey::from_sec1_bytes(&pub_key)
        .map_err(|e| format!("Failed to parse key: {}", e))?;
    let address = ethers_core::utils::public_key_to_address(&key);

    // 3. Return the checksummed address
    Ok(to_checksum(&address, None))
}

#[update]
async fn sign_eth_transaction(
    chain_id: u64,
    to: String,
    value: u64,
    data: Vec<u8>,
    nonce: u64,
    max_fee_per_gas: u64,
    max_priority_fee_per_gas: u64,
) -> Result<String, String> {
    ic_cdk::println!("Starting ETH transaction signing...");

    // 1. Get the public key
    let (pub_key_response,): (EcdsaPublicKeyResponse,) = ic_cdk::call(
        CanisterId::management_canister(),
        "ecdsa_public_key",
        (EcdsaPublicKeyArgument {
            canister_id: None,
            derivation_path: DEFAULT_DERIVATION_PATH.clone(),
            key_id: get_ecdsa_key_id(),
        },),
    )
    .await
    .map_err(|(code, msg)| format!("Failed to get ECDSA public key: {} ({:?})", msg, code))?;
    let pub_key = pub_key_response.public_key;

    // 2. Build the EIP-1559 transaction request
    let to_address = EthAddress::from_str(&to)
        .map_err(|e| format!("Invalid 'to' address: {}", e))?;

    let tx = Eip1559TransactionRequest {
        to: Some(NameOrAddress::Address(to_address)),
        from: None, // 'from' is derived from the signature
        nonce: Some(U256::from(nonce)),
        value: Some(U256::from(value)),
        data: Some(data.into()),
        gas: Some(U256::from(21000u64)), // Simple transfer. For swaps, estimate this.
        max_fee_per_gas: Some(U256::from(max_fee_per_gas)),
        max_priority_fee_per_gas: Some(U256::from(max_priority_fee_per_gas)),
        chain_id: Some(U64::from(chain_id)),
        access_list: Default::default(),
    };

    // 3. RLP-encode the transaction (unsigned, for hashing)
    let rlp_encoded_bytes = tx.rlp(); // Returns Bytes directly
    let mut rlp_encoded = vec![0x02]; // EIP-1559 type byte
    rlp_encoded.extend_from_slice(&rlp_encoded_bytes);

    let tx_hash = Keccak256::digest(&rlp_encoded).to_vec();
    ic_cdk::println!("Transaction hash to sign: {}", hex::encode(&tx_hash));

    // 4. Sign the hash with threshold ECDSA
    let cycles_for_signing = 26_153_846_153; // Get this cost from the error

    let (sign_response,): (SignWithEcdsaResponse,) = ic_cdk::api::call::call_with_payment(
        CanisterId::management_canister(),
        "sign_with_ecdsa",
        (SignWithEcdsaArgument {
            message_hash: tx_hash.clone(),
            derivation_path: DEFAULT_DERIVATION_PATH.clone(),
            key_id: get_ecdsa_key_id(),
        },),
        cycles_for_signing, // <-- THIS IS THE FIX
    )
    .await
    .map_err(|(code, msg)| format!("Failed to sign: {} ({:?})", msg, code))?;


    

    let signature = sign_response.signature;

    // 5. Reconstruct the signature (r, s, v)
    let r = U256::from_big_endian(&signature[0..32]);
    let s = U256::from_big_endian(&signature[32..64]);
    let v = calculate_recovery_id(&tx_hash, &signature, &pub_key, chain_id)
        .map_err(|e| format!("Failed to calculate recovery ID: {}", e))?;

    let eth_signature = EthSignature { r, s, v: v.as_u64() };

    // 6. RLP-encode the *signed* transaction
    // Call the correct .rlp_signed() method, which returns the encoded bytes
    let rlp_signed_bytes = tx.rlp_signed(&eth_signature);

    // Add the EIP-1559 transaction type byte (0x02) to the front
    let mut rlp_signed_encoded = vec![0x02]; 
    rlp_signed_encoded.extend_from_slice(&rlp_signed_bytes);

    // Format as a hex string and return
    let raw_tx_hex = format!("0x{}", hex::encode(rlp_signed_encoded));
    ic_cdk::println!("Signed raw transaction: {}", raw_tx_hex);

    Ok(raw_tx_hex)

}

#[update]
async fn get_btc_address() -> Result<String, String> {
    ic_cdk::println!("Requesting BTC (Schnorr) public key...");

    // 1. Get the canister's Schnorr public key (this is 32 bytes)
    let (response,): (SchnorrPublicKeyResponse,) = ic_cdk::call(
        CanisterId::management_canister(),
        "schnorr_public_key",
        (SchnorrPublicKeyArgument {
            canister_id: None,
            derivation_path: DEFAULT_DERIVATION_PATH.clone(),
            key_id: get_schnorr_key_id(),
        },),
    )
    .await
    .map_err(|(code, msg)| format!("Failed to get Schnorr public key: {} ({:?})", msg, code))?;

    let pub_key_bytes = response.public_key;
    ic_cdk::println!("Got public key hex: {}", hex::encode(&pub_key_bytes));

    // --- THIS IS THE CORRECTED LOGIC ---

    // 2. Parse the 32-byte key directly into an XOnlyPublicKey
    let x_only_key = XOnlyPublicKey::from_slice(&pub_key_bytes[1..])
        .map_err(|e| format!("Failed to parse x-only key: {}", e))?;

    // 3. Create a Secp256k1 context
    let secp = Secp256k1::new();

    // 4. Create a Taproot (P2TR) address for Bitcoin Testnet
    let address = BtcAddress::p2tr(
        &secp,
        x_only_key,           // The internal key
        None,                 // No merkle root
        BtcNetwork::Testnet,  // Network
    );

    Ok(address.to_string())
}

/// Helper to get our own pubkey and address.
async fn get_our_btc_data(secp: &Secp256k1<All>) -> Result<(XOnlyPublicKey, BtcAddress), String> {
    // 1. Get the public key
    let (response,): (SchnorrPublicKeyResponse,) = ic_cdk::call(
        CanisterId::management_canister(),
        "schnorr_public_key",
        (SchnorrPublicKeyArgument {
            canister_id: None,
            derivation_path: DEFAULT_DERIVATION_PATH.clone(),
            key_id: get_schnorr_key_id(),
        },),
    )
    .await
    .map_err(|(code, msg)| format!("Failed to get Schnorr public key: {} ({:?})", msg, code))?;

    // 2. Parse the 32-byte key
    let x_only_key = XOnlyPublicKey::from_slice(&response.public_key[1..])
        .map_err(|e| format!("Failed to parse x-only key: {}", e))?;

    // 3. Get the address
    let address = BtcAddress::p2tr(
        secp,
        x_only_key,
        None,
        bitcoin::Network::Testnet,
    );

    Ok((x_only_key, address))
}

#[update]
async fn get_utxos(address: String) -> Result<GetUtxosResponse, String> {
    ic_cdk::println!("Getting UTXOs for address: {}", address);
    
    let request = GetUtxosRequest {
        address: address.clone(),
        network: BitcoinNetwork::Testnet,
        filter: None, // No filter, get all UTXOs
    };

    let (response,): (GetUtxosResponse,) = ic_cdk::call(
        CanisterId::management_canister(),
        "bitcoin_get_utxos",
        (request,),
    )
    .await
    .map_err(|(code, msg)| format!("Failed to get UTXOs for {}: {} ({:?})", address, msg, code))?;

    ic_cdk::println!("Got {} UTXOs.", response.utxos.len());
    Ok(response)
}

// --- 4. CANDID EXPORT ---
ic_cdk::export_candid!();

// --- ADD THIS BLOCK AT THE VERY END OF THE FILE ---

/// A custom getrandom implementation for Wasm.
/// This is required by crypto crates like ethers-core and k256.
#[no_mangle]
fn getrandom(_buf: *mut u8, _len: usize) -> i32 {
    ic_cdk::trap("getrandom() is not implemented. For secure randomness, use ic_cdk::api::management_canister::main::raw_rand().");
    // This function should trap, so we don't return.
}