use risc0_zkvm::{default_prover, ExecutorEnv, Receipt};
use serde_json;
mod types;
use types::RiscZeroProof;

#[doc(hidden)]
pub fn verify<T: AsRef<[u8]>>(
    proof_serialized: T
) -> [u8;1]{
    let risc0_proof: RiscZeroProof = serde_json::from_slice(&proof_serialized.as_ref()).unwrap();
    let program_id: [u32;8] = risc0_proof.program_id.try_into().unwrap();
    match risc0_proof.receipt.verify(program_id){
        Ok(_) => [1],
        Err(_) => [0]
    }
}