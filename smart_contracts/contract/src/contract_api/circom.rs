use crate::{ext_ffi, unwrap_or_revert::UnwrapOrRevert};
use casper_types::api_error;
use alloc::{collections::BTreeSet, vec, vec::Vec, string::String};

pub fn circom_verifier<T: AsRef<[u8]>>(circuit_bytes: T, proof_points: T, inputs: T, gamma_abc_g1: T) -> Vec<u8>{
    let mut res: Vec<u8> = Vec::new();
    let result = unsafe {
        ext_ffi::casper_circom_verifier(
            circuit_bytes.as_ref().as_ptr(),
            1,
            proof_points.as_ref().as_ptr(),
            1,
            inputs.as_ref().as_ptr(),
            1,
            gamma_abc_g1.as_ref().as_ptr(),
            1,
            res.as_mut_ptr(),
            1
        )
    };
    api_error::result_from(result).unwrap_or_revert();
    res
}