use ic_cdk::api::management_canister::main::CanisterId;
use candid::{CandidType, Principal};
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};

// ============================================================================
// VETKEYS TYPES
// ============================================================================

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub enum VetKDCurve {
    #[serde(rename = "bls12_381")]
    Bls12_381,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct VetKDKeyId {
    pub curve: VetKDCurve,
    pub name: String,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct VetKDPublicKeyArgs {
    pub canister_id: Option<Principal>,
    pub context: Vec<u8>,
    pub key_id: VetKDKeyId,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct VetKDPublicKeyResponse {
    pub public_key: Vec<u8>,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct VetKDDeriveKeyArgs {
    pub input: Vec<u8>,
    pub context: Vec<u8>,
    pub transport_public_key: Vec<u8>,
    pub key_id: VetKDKeyId,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct VetKDDeriveKeyResponse {
    pub encrypted_key: Vec<u8>,
}

// ============================================================================
// CONFIGURATION
// ============================================================================

/// Domain separator for VEIL batch auction
const VEIL_DOMAIN_SEPARATOR: &[u8] = b"VEIL-BATCH-AUCTION-V1";

/// Get the vetKD key ID
/// For mainnet: use "key_1" (production key)
/// For local/testnet: use "test_key_1" 
fn get_vetkd_key_id() -> VetKDKeyId {
    VetKDKeyId {
        curve: VetKDCurve::Bls12_381,
        name: if cfg!(feature = "mainnet") {
            "key_1".to_string()
        } else {
            "test_key_1".to_string()
        },
    }
}

// ============================================================================
// PUBLIC KEY MANAGEMENT
// ============================================================================

/// Get the master public key for IBE/Timelock encryption
/// This key is used by the frontend to encrypt orders
pub async fn get_encryption_public_key() -> Result<Vec<u8>, String> {
    ic_cdk::println!("Fetching vetKD master public key...");
    
    let request = VetKDPublicKeyArgs {
        canister_id: None, // Use current canister's key
        context: VEIL_DOMAIN_SEPARATOR.to_vec(),
        key_id: get_vetkd_key_id(),
    };

    let (response,): (VetKDPublicKeyResponse,) = ic_cdk::call(
        CanisterId::management_canister(),
        "vetkd_public_key",
        (request,),
    )
    .await
    .map_err(|(code, msg)| {
        format!("Failed to get vetKD public key: {} ({:?})", msg, code)
    })?;

    ic_cdk::println!("Got master public key: {} bytes", response.public_key.len());
    Ok(response.public_key)
}

// ============================================================================
// TIMELOCK ENCRYPTION (for MEV protection)
// ============================================================================

/// Generate timelock encryption identity for a specific round
/// This allows encrypting orders that can only be decrypted after the round ends
pub fn generate_timelock_identity(round_id: u64) -> Vec<u8> {
    // Use round_id as the timelock identity
    // All orders in the same round use the same identity
    // This enables batch decryption after the timelock expires
    let mut identity = Vec::new();
    identity.extend_from_slice(b"ROUND:");
    identity.extend_from_slice(&round_id.to_be_bytes());
    identity
}

/// Derive the decryption key for a specific round (after timelock expires)
/// This is called by the canister after the round submission period ends
pub async fn derive_round_decryption_key(
    round_id: u64,
    transport_public_key: Vec<u8>,
) -> Result<Vec<u8>, String> {
    ic_cdk::println!("Deriving decryption key for round {}", round_id);

    let timelock_identity = generate_timelock_identity(round_id);

    let request = VetKDDeriveKeyArgs {
        input: timelock_identity,
        context: VEIL_DOMAIN_SEPARATOR.to_vec(),
        transport_public_key,
        key_id: get_vetkd_key_id(),
    };

    // Cost for vetKD key derivation (from ICP docs)
    let cycles_needed = 10_000_000_000u128; // 10B cycles

    let (response,): (VetKDDeriveKeyResponse,) = ic_cdk::api::call::call_with_payment128(
        CanisterId::management_canister(),
        "vetkd_derive_key",
        (request,),
        cycles_needed,
    )
    .await
    .map_err(|(code, msg)| {
        format!("Failed to derive vetKey: {} ({:?})", msg, code)
    })?;

    ic_cdk::println!("Derived encrypted key: {} bytes", response.encrypted_key.len());
    Ok(response.encrypted_key)
}

// ============================================================================
// ORDER ENCRYPTION/DECRYPTION
// ============================================================================

/// Generate a commitment hash for an order (before encryption)
/// This creates a binding commitment that proves the order wasn't tampered with
pub fn generate_commitment_hash(order_data: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(order_data.as_bytes());
    hex::encode(hasher.finalize())
}

/// Verify that an order's commitment hash matches its decrypted content
pub fn verify_commitment(
    decrypted_data: &str,
    commitment_hash: &str,
) -> Result<bool, String> {
    let computed_hash = generate_commitment_hash(decrypted_data);
    Ok(computed_hash == commitment_hash)
}

/// Decrypt a batch of orders for a specific round
/// This is called after the timelock expires (round submission ends)
/// 
/// WORKFLOW:
/// 1. Round submission period ends
/// 2. Canister derives the round-specific decryption key using vetKD
/// 3. All orders encrypted with that round's timelock can now be decrypted
/// 4. Verify commitment hashes to ensure orders weren't tampered with
/// 5. Return decrypted orders for clearing
pub async fn decrypt_order_batch(
    orders: Vec<crate::types::Order>,
) -> Result<Vec<crate::types::Order>, String> {
    if orders.is_empty() {
        return Ok(Vec::new());
    }

    let round_id = orders[0].round_id;
    ic_cdk::println!("Decrypting {} orders for round {}", orders.len(), round_id);

    // For production with vetKeys:
    // 1. Generate transport key pair in canister
    // 2. Call derive_round_decryption_key with transport public key
    // 3. Decrypt the encrypted_key response with transport private key
    // 4. Use resulting symmetric key to decrypt order payloads
    
    // For hackathon demo (vetKeys API may not be fully available):
    // We simulate the process by verifying commitment hashes
    // The encrypted_payload contains the order details in JSON format
    
    let mut decrypted_orders = Vec::new();
    
    for order in orders {
        // Simulate decryption by parsing the payload
        // In production, this would use AES-GCM with the derived vetKey
        let decrypted_data = String::from_utf8(order.encrypted_payload.clone())
            .map_err(|e| format!("Invalid UTF-8 in order {}: {}", order.id, e))?;

        // CRITICAL: Verify commitment hash
        if !verify_commitment(&decrypted_data, &order.commitment_hash)? {
            return Err(format!(
                "SECURITY VIOLATION: Commitment hash mismatch for order {}. \
                Order may have been tampered with!",
                order.id
            ));
        }

        ic_cdk::println!("✓ Order {} verified (commitment hash matches)", order.id);
        decrypted_orders.push(order);
    }

    ic_cdk::println!(
        "Successfully decrypted and verified {} orders",
        decrypted_orders.len()
    );

    Ok(decrypted_orders)
}

// ============================================================================
// IDENTITY-BASED ENCRYPTION (for user-specific keys)
// ============================================================================

/// Derive a user-specific decryption key
/// This allows users to encrypt data to their own Principal
/// Useful for private notes, settings, or personal order history
pub async fn derive_user_key(
    user: Principal,
    transport_public_key: Vec<u8>,
) -> Result<Vec<u8>, String> {
    ic_cdk::println!("Deriving user-specific key for {}", user);

    let request = VetKDDeriveKeyArgs {
        input: user.as_slice().to_vec(),
        context: VEIL_DOMAIN_SEPARATOR.to_vec(),
        transport_public_key,
        key_id: get_vetkd_key_id(),
    };

    let cycles_needed = 10_000_000_000u128;

    let (response,): (VetKDDeriveKeyResponse,) = ic_cdk::api::call::call_with_payment128(
        CanisterId::management_canister(),
        "vetkd_derive_key",
        (request,),
        cycles_needed,
    )
    .await
    .map_err(|(code, msg)| {
        format!("Failed to derive user key: {} ({:?})", msg, code)
    })?;

    Ok(response.encrypted_key)
}

// ============================================================================
// CANISTER ENDPOINTS
// ============================================================================

/// Get the timelock identity for a specific round
/// Frontend uses this with the master public key to encrypt orders
#[ic_cdk_macros::query]
pub fn get_round_timelock_identity(round_id: u64) -> Vec<u8> {
    generate_timelock_identity(round_id)
}

/// Derive encrypted user key for the caller
/// Used for user-specific encrypted data storage
#[ic_cdk_macros::update]
pub async fn get_my_encrypted_key(
    transport_public_key: Vec<u8>,
) -> Result<Vec<u8>, String> {
    let caller = ic_cdk::caller();
    
    // Prevent anonymous access
    if caller == Principal::anonymous() {
        return Err("Anonymous users cannot derive keys".to_string());
    }

    derive_user_key(caller, transport_public_key).await
}

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

/// Format order data for encryption
/// This creates a JSON string that will be encrypted and stored
pub fn format_order_for_encryption(
    order_type: &crate::types::OrderType,
    asset: &crate::types::Asset,
    amount: u64,
    price_limit: u64,
) -> String {
    serde_json::json!({
        "order_type": format!("{:?}", order_type),
        "asset": format!("{:?}", asset),
        "amount": amount,
        "price_limit": price_limit,
        "timestamp": ic_cdk::api::time(),
    })
    .to_string()
}

/// Parse decrypted order data
pub fn parse_decrypted_order(data: &str) -> Result<serde_json::Value, String> {
    serde_json::from_str(data)
        .map_err(|e| format!("Failed to parse decrypted order: {}", e))
}
