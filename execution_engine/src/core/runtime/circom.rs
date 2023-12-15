/// Crate with elliptic curve types from arkwork's circom-compat & deps
pub mod types;
use ark_serialize::CanonicalDeserialize;
use types::CircomProof;
use ark_groth16::{Groth16, Proof, PreparedVerifyingKey};
use ark_crypto_primitives::snark::SNARK;
use ark_ec::bls12::Bls12;
use ark_bls12_377::{Bls12_377, Config};
use serde_json;

type GrothBn = Groth16<Bls12_377>;

#[doc(hidden)]
pub fn verify<T: AsRef<[u8]>>(
    proof: T
) -> [u8;1]{
    let circom_proof: CircomProof = serde_json::from_slice(&proof.as_ref()).unwrap(); 
    let deserialized_inputs = Vec::deserialize_uncompressed(&mut circom_proof.inputs.as_slice()).unwrap();
    let deserialized_proof: Proof<Bls12<Config>> = Proof::deserialize_uncompressed(&mut circom_proof.proof.as_slice()).unwrap();
    let deserialized_vk: PreparedVerifyingKey<Bls12<Config>> = PreparedVerifyingKey::deserialize_uncompressed(&mut circom_proof.vk.as_slice()).unwrap(); 
    // verify groth16 proof
    if GrothBn::verify_with_processed_vk(&deserialized_vk, &deserialized_inputs, &deserialized_proof).unwrap() == true{
        [1u8]
    }
    else{
        [0u8]
    }
}