use pocket_ic::PocketIc;
use candid::{Encode, Decode, Principal};
use sha2::{Sha256, Digest};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use aes_gcm::aead::{Aead, KeyInit};

fn sha256(data: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(data);
    hex::encode(h.finalize())
}

#[test]
fn full_vetkd_round() {
    let ic = PocketIc::new();

    // ---------- Deploy VetKeys Engine ----------
    let vetkeys_wasm = std::fs::read(
        "vetkeys_engine/target/wasm32-unknown-unknown/release/vetkeys_engine.wasm"
    ).unwrap();

    let vetkeys_id = ic.create_canister();
    ic.install_canister(vetkeys_id, vetkeys_wasm, vec![], None);

    // ---------- Deploy Backend ----------
    let backend_wasm = std::fs::read(
        "target/wasm32-unknown-unknown/release/mempool_chess_backend.wasm"
    ).unwrap();

    let backend_id = ic.create_canister();
    ic.install_canister(backend_id, backend_wasm, vec![], None);

    // ---------- Inject VetKeys ID ----------
    let inject_args = Encode!(&Principal::from_slice(vetkeys_id.as_slice())).unwrap();

    ic.update_call(
        backend_id,
        Principal::anonymous(),
        "set_vetkeys_engine_id",
        inject_args,
    ).unwrap();

    // ---------- 1) Fetch public key ----------
    let resp = ic.query_call(
        backend_id,
        Principal::anonymous(),
        "get_encryption_public_key",
        Encode!().unwrap(),
    ).unwrap();

    let pk: Vec<u8> = Decode!(&resp, Vec<u8>).unwrap();
    assert!(!pk.is_empty());

    // ---------- 2) Derive AES key ----------
    let key_hash = sha256(&pk);
    let key_bytes = hex::decode(key_hash).unwrap();
    let aes_key = Key::<Aes256Gcm>::from_slice(&key_bytes[..32]);
    let cipher = Aes256Gcm::new(aes_key);

    // ---------- 3) Encrypt order ----------
    let plaintext = br#"{"order":"BUY"}"#;
    let nonce_bytes = [0u8; 12];
    let nonce = Nonce::from_slice(&nonce_bytes);

    let encrypted = cipher.encrypt(nonce, plaintext.as_ref()).unwrap();
    let commitment = sha256(plaintext);

    // ---------- 4) Submit order ----------
    let args = Encode!(&1u64, &encrypted, &commitment).unwrap();

    ic.update_call(
        backend_id,
        Principal::anonymous(),
        "pocketic_submit_order",
        args,
    ).unwrap();

    // ---------- 5) Read stored ciphertext ----------
    let stored_raw = ic.query_call(
        backend_id,
        Principal::anonymous(),
        "pocketic_get_order_ciphertext",
        Encode!().unwrap(),
    ).unwrap();

    let stored: Vec<u8> = Decode!(&stored_raw, Vec<u8>).unwrap();

    // ---------- 6) Decrypt ----------
    let decrypted = cipher.decrypt(nonce, stored.as_slice()).unwrap();
    assert_eq!(&decrypted, plaintext);

    println!("✅ Full PocketIC VetKD flow passed");
}
