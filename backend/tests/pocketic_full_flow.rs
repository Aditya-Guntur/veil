use pocket_ic::PocketIc;
use candid::{Encode, Decode, Principal};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use aes_gcm::aead::{Aead, KeyInit};
use sha2::{Sha256, Digest};

fn sha256(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

#[test]
fn pocketic_full_encrypted_flow() {
    let ic = PocketIc::new();

    // === Load backend wasm
    let wasm = std::fs::read(
        "target/wasm32-unknown-unknown/release/mempool_chess_backend.wasm"
    ).expect("Run cargo build --target wasm32-unknown-unknown --release first");

    // === Create canister
    let canister = ic.create_canister();
    ic.add_cycles(canister, 10_000_000_000_000u128);
    ic.install_canister(canister, wasm, vec![], None);

    let sender = Principal::anonymous();

    // === Call backend: get encryption public key (demo mode)
    let resp = ic.query_call(
        canister,
        sender,
        "get_encryption_public_key",
        Encode!().unwrap()
    ).unwrap();

    let pk: Vec<u8> = Decode!(&resp, Vec<u8>).unwrap();

    assert!(!pk.is_empty());

    // === Deterministic mock AES key
    let hash = sha256(pk.as_slice());
    let key_bytes = hex::decode(hash).unwrap();
    let aes_key = Key::<Aes256Gcm>::from_slice(&key_bytes[..32]);
    let cipher = Aes256Gcm::new(aes_key);

    // === Payload
    let plaintext = br#"{
        "order_type":"Buy",
        "asset":"BTC",
        "amount":100000,
        "price_limit":9000
    }"#;

    let nonce_bytes = [1u8; 12];
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher.encrypt(nonce, plaintext.as_ref()).unwrap();
    let commitment = sha256(plaintext);

    // === Submit order
    let args = Encode!(
        &1u64,
        &ciphertext,
        &commitment
    ).unwrap();

    ic.update_call(
        canister,
        sender,
        "pocketic_submit_order",
        args
    ).unwrap();

    // === Read back stored ciphertext
    let stored_raw = ic.query_call(
        canister,
        sender,
        "pocketic_get_order_ciphertext",
        Encode!().unwrap()
    ).unwrap();

    let stored: Vec<u8> = Decode!(&stored_raw, Vec<u8>).unwrap();

    // === Decrypt
    let decrypted = cipher.decrypt(nonce, stored.as_slice()).unwrap();
    let new_hash = sha256(&decrypted);

    assert_eq!(new_hash, commitment);

    println!("✅ PocketIC vetKD-style demo flow passed");
}
