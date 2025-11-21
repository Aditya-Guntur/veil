use candid::{CandidType, Deserialize, Principal};
use ic_cdk_macros::*;
use serde::Serialize;

// ==============================
// Types
// ==============================

#[derive(CandidType, Serialize, Deserialize, Clone)]
pub enum VetKDCurve {
    #[serde(rename = "bls12_381")]
    Bls12_381,
}

#[derive(CandidType, Serialize, Deserialize, Clone)]
pub struct VetKDKeyId {
    pub curve: VetKDCurve,
    pub name: String,
}

#[derive(CandidType, Serialize, Deserialize)]
pub struct VetKDPublicKeyArgs {
    pub canister_id: Option<Principal>,
    pub context: Vec<u8>,
    pub key_id: VetKDKeyId,
}

#[derive(CandidType, Serialize, Deserialize)]
pub struct VetKDPublicKeyResponse {
    pub public_key: Vec<u8>,
}

#[derive(CandidType, Serialize, Deserialize)]
pub struct VetKDDeriveKeyArgs {
    pub input: Vec<u8>,
    pub context: Vec<u8>,
    pub transport_public_key: Vec<u8>,
    pub key_id: VetKDKeyId,
}

#[derive(CandidType, Serialize, Deserialize)]
pub struct VetKDDeriveKeyResponse {
    pub encrypted_key: Vec<u8>,
}

// ==============================
// Config
// ==============================

const VEIL_DOMAIN_SEPARATOR: &[u8] = b"VEIL-BATCH-AUCTION-V1";

fn key_id() -> VetKDKeyId {
    VetKDKeyId {
        curve: VetKDCurve::Bls12_381,
        name: "test_key_1".to_string(),
    }
}

// ==============================
// Public API (mock vetKD for PocketIC)
// ==============================

#[update]
fn get_public_key() -> Vec<u8> {
    // PocketIC has no real vetKD
    let mut fake = vec![0u8; 32];
    fake[0..4].copy_from_slice(b"VTKD");
    fake
}

#[update]
fn derive_round_key(round_id: u64) -> Vec<u8> {
    let mut out = vec![0u8; 32];
    out[..8].copy_from_slice(&round_id.to_be_bytes());
    out
}

#[update]
fn derive_user_key(user: Principal) -> Vec<u8> {
    let mut out = vec![0u8; 32];
    let u = user.as_slice();
    let n = core::cmp::min(u.len(), 32);
    out[..n].copy_from_slice(&u[..n]);
    out
}

ic_cdk::export_candid!();
