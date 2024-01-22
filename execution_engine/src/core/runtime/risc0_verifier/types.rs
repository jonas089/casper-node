use serde::{Serialize, Deserialize};
use risc0_zkvm::Receipt;

#[derive(Serialize, Deserialize)]
pub struct RiscZeroProof{
    pub receipt: Receipt,
    pub program_id: Vec<u32>
}