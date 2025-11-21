use pocket_ic::PocketIc;
use candid::{Encode, Decode};
use candid::Principal;
use aes_gcm::{Aes256Gcm, Key, Nonce};
use aes_gcm::aead::{Aead, KeyInit};
use sha2::{Sha256, Digest};


#[derive(candid::CandidType, candid::Deserialize)]
enum ResultBytes {
    Ok(Vec<u8>),
    Err(String),
}

// ========================
// HELPERS
// ========================

fn sha256(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

// ========================
// TEST
// ========================

#[test]
fn full_vetkd_encrypted_round() {
    let ic = PocketIc::new();

    // read wasm
    let wasm = std::fs::read(
        "target/wasm32-unknown-unknown/release/mempool_chess_backend.wasm"
    ).expect("Build backend first");

    // create & install canister
    let canister = ic.create_canister();
    ic.add_cycles(canister, 10_000_000_000_000u128);
    ic.install_canister(canister, wasm, vec![], None);

    let sender = Principal::anonymous();

    // ============================
    // 1. Get Encryption Public Key
    // ============================

    let resp = ic.update_call(
        canister,
        sender,
        "get_encryption_public_key",
        Encode!().unwrap(),
    ).unwrap();

    let res: ResultBytes = Decode!(&resp, ResultBytes).unwrap();

    let pk = match res {
        ResultBytes::Ok(bytes) => bytes,
        ResultBytes::Err(e) => panic!("Canister returned error: {}", e),
    };

    assert!(!pk.is_empty());

    // ============================
    // 2. Derive AES Key (demo)
    // ============================

    let round_key_raw = sha256(pk.as_slice());
    let key_bytes = hex::decode(round_key_raw).unwrap();
    let aes_key = Key::<Aes256Gcm>::from_slice(&key_bytes[..32]);
    let cipher = Aes256Gcm::new(aes_key);

    // ============================
    // 3. Create Order
    // ============================

    let plaintext = br#"{
        "order_type": "Buy",
        "asset": "BTC",
        "amount": 100000,
        "price_limit": 8500
    }"#;

    let nonce_bytes = [0u8; 12];
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher.encrypt(nonce, plaintext.as_ref()).unwrap();
    let commitment = sha256(plaintext);

    // ============================
    // 4. Submit Order
    // ============================

    let args = Encode!(
        &1u64,
        &ciphertext,
        &commitment
    ).unwrap();

    ic.update_call(
        canister,
        sender,
        "pocketic_submit_order",
        args,
    ).unwrap();

    // ============================
    // 5. Read Back Ciphertext
    // ============================

    let stored_raw = ic.query_call(
        canister,
        sender,
        "pocketic_get_order_ciphertext",
        Encode!().unwrap(),
    ).unwrap();

    let stored: Vec<u8> = Decode!(&stored_raw, Vec<u8>).unwrap();

    // ============================
    // 6. Decrypt
    // ============================

    let decrypted = cipher.decrypt(nonce, stored.as_slice()).unwrap();

    // ============================
    // 7. Verify Hash
    // ============================

    let new_hash = sha256(&decrypted);
    assert_eq!(new_hash, commitment);

    println!("✅ Full AES-GCM PocketIC flow succeeded");
}
