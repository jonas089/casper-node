use serde::{Serialize, Deserialize};

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct NoirProof {
    pub verifier: Vec<u8>,
    pub proof: Vec<u8>,
}