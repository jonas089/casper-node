mod types;
use std::io::{BufReader, Cursor};

use types::{CircomProof, Groth16Proof, Groth16VerifyingKey};

use ark_groth16::{Groth16, ProvingKey};
use ark_crypto_primitives::snark::SNARK;
use num_bigint::BigInt;
use ark_ec::{
    bn::Bn
};
use ark_serialize::{CanonicalSerialize, CanonicalDeserialize};
use ark_circom::ethereum::{Proof, VerifyingKey};
use ark_circom::{CircomConfig, CircomBuilder, CircomCircuit};
use ark_bn254::{Bn254, Config, G1Affine, G2Affine};
use ark_circom::{circom::R1CSFile, WitnessCalculator};
extern crate tempfile;
use tempfile::NamedTempFile;

type GrothBn = Groth16<Bn254>;

#[doc(hidden)]
pub fn verify(
    circuit_bytes: Vec<Vec<u8>>,
    proof_points: Vec<Vec<u8>>,
    inputs: Vec<(String, Vec<i32>)>,
    gamma_abc_g1: Vec<Vec<u8>>
) -> bool{
    let vk: ark_groth16::VerifyingKey<Bn<Config>> = Groth16VerifyingKey { 
        alpha_g1: proof_points[3].clone(), 
        beta_g2: proof_points[4].clone(), 
        delta_g2: proof_points[5].clone(),
        gamma_g2: proof_points[6].clone(), 
        gamma_abc_g1: gamma_abc_g1
    }.build();
    let proof: Groth16Proof = Groth16Proof{
        a: proof_points[0].clone(),
        b: proof_points[1].clone(),
        c: proof_points[2].clone()
    };
    let pvk: ark_groth16::PreparedVerifyingKey<Bn<Config>> = GrothBn::process_vk(&vk).unwrap();
    let mut wasm_file = NamedTempFile::new().unwrap();
    let mut r1cs_file = NamedTempFile::new().unwrap();
    let _ = wasm_file.write_all(&circuit_bytes[0]);
    let _ = r1cs_file.write_all(&circuit_bytes[1]);
    wasm_file.flush().unwrap();
    r1cs_file.flush().unwrap();
    let wasm_path: tempfile::TempPath = wasm_file.into_temp_path();
    let r1cs_path: tempfile::TempPath = r1cs_file.into_temp_path();
    let cfg = CircomConfig::<Bn254>::new(
        wasm_path,
        r1cs_path
    ).unwrap();
    // Insert our public inputs as key value pairs
    let mut builder: CircomBuilder<Bn<Config>> = CircomBuilder::new(cfg);
    if inputs.len() > 0{
        for (key, value) in inputs{
            builder.push_input(key, value);
        };
    }
    
    let circom: CircomCircuit<Bn<Config>> = builder.build().unwrap();
    let inputs = circom.get_public_inputs().unwrap();
    // verify groth16 proof
    GrothBn::verify_with_processed_vk(&pvk, &inputs, &proof.build()).unwrap()
}