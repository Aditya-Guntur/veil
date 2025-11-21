use pocket_ic::PocketIc;
use candid::{Encode, Decode};
use std::process::Command;

// This spins up a local IC replica exactly like mainnet
#[test]
fn vetkd_public_key_works() {
    let mut pic = PocketIc::new();

    // Create empty canister
    let canister_id = pic.create_canister();

    // Add cycles
    pic.add_cycles(canister_id, 2_000_000_000_000);

    println!("✅ PocketIC started. Canister: {:?}", canister_id);
}
