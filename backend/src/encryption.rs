use candid::Principal;
use crate::ResultBytes;

// =====================================
// FLAGS
// =====================================
#[inline]
fn is_demo() -> bool {
    cfg!(feature = "demo")
}

#[inline]
fn is_vetkeys_enabled() -> bool {
    cfg!(feature = "vetkeys")
}

// ============================================================================
// Global mutable for PocketIC tests
// ============================================================================

static mut VETKEYS_ENGINE_ID: Option<Principal> = None;

pub fn set_vetkeys_engine_canister_id(id: Principal) {
    unsafe {
        VETKEYS_ENGINE_ID = Some(id);
    }
}

fn vetkeys_engine_canister_id() -> Principal {
    unsafe {
        VETKEYS_ENGINE_ID.expect("VetKeys engine canister id not set")
    }
}

// ============================================================================
// CROSS-CANISTER VETKEYS BRIDGE
// ============================================================================

pub async fn get_encryption_public_key() -> Result<Vec<u8>, String> {
    if is_demo() {
        let mut fake_key = vec![0u8; 32];
        fake_key[..4].copy_from_slice(b"DEMO");
        return Ok(fake_key);
    }

    let canister = crate::vetkeys_engine_canister_id();

    let (res,): (Result<Vec<u8>, String>,) = ic_cdk::call(
        canister,
        "get_encryption_public_key",
        (),
    )
    .await
    .map_err(|e| format!("Call failed: {:?}", e))?;

    res
}

pub async fn derive_round_decryption_key(
    round_id: u64,
    transport_public_key: Vec<u8>,
) -> Result<Vec<u8>, String> {
    if is_demo() {
        let mut fake_key = vec![0u8; 32];
        fake_key[..8].copy_from_slice(&round_id.to_be_bytes());
        return Ok(fake_key);
    }

    let canister = crate::vetkeys_engine_canister_id();

    let (res,): (Result<Vec<u8>, String>,) = ic_cdk::call(
        canister,
        "derive_round_decryption_key",
        (round_id, transport_public_key),
    )
    .await
    .map_err(|e| format!("Call failed: {:?}", e))?;

    res
}


pub async fn derive_user_key(
    user: Principal,
    transport_public_key: Vec<u8>,
) -> Result<Vec<u8>, String> {
    if is_demo() {
        let mut fake_key = vec![0u8; 32];
        let user_bytes = user.as_slice();
        let copy_len = core::cmp::min(user_bytes.len(), 32);
        fake_key[..copy_len].copy_from_slice(&user_bytes[..copy_len]);
        return Ok(fake_key);
    }

    let canister = crate::vetkeys_engine_canister_id();

    let (res,): (Result<Vec<u8>, String>,) = ic_cdk::call(
        canister,
        "derive_user_key",
        (user, transport_public_key),
    )
    .await
    .map_err(|e| format!("Call failed: {:?}", e))?;

    res
}

// =====================================
// TIMELOCK
// =====================================

pub fn generate_timelock_identity(round_id: u64) -> Vec<u8> {
    let mut identity = Vec::new();
    identity.extend_from_slice(b"ROUND:");
    identity.extend_from_slice(&round_id.to_be_bytes());
    identity
}

// =====================================
// COMMITMENT
// =====================================

use sha2::{Digest, Sha256};

pub fn generate_commitment_hash(order_data: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(order_data.as_bytes());
    hex::encode(hasher.finalize())
}

pub fn verify_commitment(
    decrypted_data: &str,
    commitment_hash: &str,
) -> Result<bool, String> {
    if is_demo() {
        return Ok(true);
    }

    let computed = generate_commitment_hash(decrypted_data);
    if computed != commitment_hash {
        return Err("SECURITY VIOLATION: Commitment hash mismatch".to_string());
    }

    Ok(true)
}

pub async fn decrypt_order_batch(
    orders: Vec<crate::types::Order>,
) -> Result<Vec<crate::types::Order>, String> {
    if orders.is_empty() {
        return Ok(Vec::new());
    }

    let round_id = orders[0].round_id;
    ic_cdk::println!("Decrypting {} orders for round {}", orders.len(), round_id);

    let mut decrypted_orders = Vec::new();

    for order in orders {
        // ============================
        // DEMO MODE — bypass real crypto
        // ============================
        if cfg!(feature = "demo") {
            ic_cdk::println!(
                "⚠️ DEMO MODE: Skipping decryption + commitment check for order {}",
                order.id
            );
            decrypted_orders.push(order);
            continue;
        }

        // ============================
        // REAL MODE (VetKeys)
        // ============================
        let decrypted_data = String::from_utf8(order.encrypted_payload.clone())
            .map_err(|e| format!("Invalid UTF-8 in order {}: {}", order.id, e))?;

        if !verify_commitment(&decrypted_data, &order.commitment_hash)? {
            return Err(format!(
                "SECURITY VIOLATION: Commitment hash mismatch for order {}",
                order.id
            ));
        }

        decrypted_orders.push(order);
    }

    Ok(decrypted_orders)
}
