use serde::{Serialize, Deserialize};

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct NoirProof {
    pub verifier: String,
    pub proof: String,
}