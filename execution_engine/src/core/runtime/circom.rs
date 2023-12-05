mod types;
use std::io::prelude::*;
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
use serde::{Serialize, Deserialize};
use serde_json;

type GrothBn = Groth16<Bn254>;


#[derive(Serialize, Deserialize)]
struct CircomInput{
    alpha_g1: Vec<u8>,
    beta_g2: Vec<u8>,
    delta_g2: Vec<u8>,
    gamma_g2: Vec<u8>,
    gamma_abc_g1: Vec<Vec<u8>>,
    a: Vec<u8>,
    b: Vec<u8>,
    c: Vec<u8>,
    circuit_wasm: Vec<u8>,
    circuit_r1cs: Vec<u8>,
    inputs: Vec<(String, i32)>
}


#[doc(hidden)]
pub fn verify<T: AsRef<[u8]>>(
    circom_input: T
) -> [u8;1]{
    let input: CircomInput = serde_json::from_slice(&circom_input).unwrap(); 
    let vk: ark_groth16::VerifyingKey<Bn<Config>> = Groth16VerifyingKey { 
        alpha_g1: input.alpha_g1,
        beta_g2: input.beta_g2, 
        delta_g2: input.delta_g2,
        gamma_g2: input.gamma_g2, 
        gamma_abc_g1: input.gamma_abc_g1
    }.build();
    let proof: Groth16Proof = Groth16Proof{
        a: input.a,
        b: input.b,
        c: input.c
    };
    let pvk: ark_groth16::PreparedVerifyingKey<Bn<Config>> = GrothBn::process_vk(&vk).unwrap();
    let mut wasm_file = NamedTempFile::new().unwrap();
    let mut r1cs_file = NamedTempFile::new().unwrap();
    let _ = wasm_file.write_all(&input.circuit_wasm);
    let _ = r1cs_file.write_all(&input.circuit_r1cs);
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
    if input.inputs.len() > 0{
        for (key, value) in input.inputs{
            builder.push_input(key, value);
        };
    }
    
    let circom: CircomCircuit<Bn<Config>> = builder.build().unwrap();
    let inputs = circom.get_public_inputs().unwrap();
    // verify groth16 proof
    if GrothBn::verify_with_processed_vk(&pvk, &inputs, &proof.build()).unwrap() == true{
        [1u8]
    }
    else{
        [0u8]
    }
}