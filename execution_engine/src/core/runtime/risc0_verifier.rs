use risc0_zkvm::{default_prover, ExecutorEnv, Receipt};
use kairos_risc0_types::RiscZeroProof;
use serde_json;

#[doc(hidden)]
pub fn verify<T: AsRef<[u8]>>(
    proof_serialized: T
) -> [u8;1]{
    let risc0_proof: RiscZeroProof = serde_json::from_slice(&proof_serialized.as_ref()).unwrap();
    let receipt: Receipt = serde_json::from_slice(&risc0_proof.receipt_serialized).unwrap();
    let program_id: [u32;8] = risc0_proof.program_id.try_into().unwrap();
    match receipt.verify(program_id){
        Ok(_) => [1],
        Err(_) => [0]
    }
}