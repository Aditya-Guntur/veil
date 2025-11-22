use pocket_ic::PocketIc;
use candid::{Encode, Decode};
use candid::Principal;

#[test]
fn full_vetkd_encrypted_round() {
    let ic = PocketIc::new();

    // ============ LOAD WASMS ============

    let backend_wasm = std::fs::read(
        "target/wasm32-unknown-unknown/release/mempool_chess_backend.wasm"
    ).expect("Build backend first");

    let vetkd_wasm = std::fs::read(
        "target/wasm32-unknown-unknown/release/vetkeys_engine.wasm"
    ).expect("Build vetkeys_engine first");

    // ============ INSTALL VETKD ============

    let vetkd_id = ic.create_canister();
    ic.install_canister(vetkd_id, vetkd_wasm, vec![], None);

    // ============ INSTALL BACKEND ============

    let backend_id = ic.create_canister();
    ic.install_canister(backend_id, backend_wasm, vec![], None);

    let sender = Principal::anonymous();

    // ============ CONNECT THEM ============

    ic.update_call(
        backend_id,
        sender,
        "set_vetkd_canister",
        Encode!(&vetkd_id).unwrap(),
    ).unwrap();

    // ============ NOW REAL CALL WORKS ============

    let resp = ic.query_call(
        backend_id,
        sender,
        "get_encryption_public_key",
        Encode!().unwrap(),
    ).unwrap();

    let res: Result<Vec<u8>, String> =
        Decode!(&resp, Result<Vec<u8>, String>).unwrap();

    let pk = res.unwrap();
    assert!(!pk.is_empty());

    println!("✅ vetKD real public key fetched");
}
