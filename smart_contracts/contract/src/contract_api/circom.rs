use crate::{ext_ffi, unwrap_or_revert::UnwrapOrRevert};
use casper_types::api_error;
use alloc::{collections::BTreeSet, vec, vec::Vec};

pub fn circom_verifier() -> Vec<u8>{
    let mut circuit_bytes_ptr: Vec<u8> = Vec::new();
    let mut proof_points_ptr: Vec<u8> = Vec::new();
    let mut inputs_ptr: Vec<u8> = Vec::new();
    let mut gamma_abc_g1_ptr: Vec<u8> = Vec::new();
    let mut res: Vec<u8> = Vec::new();
    let result = unsafe {
        ext_ffi::casper_circom_verifier(
            circuit_bytes_ptr.as_mut_ptr(),
            1,
            proof_points_ptr.as_mut_ptr(),
            1,
            inputs_ptr.as_mut_ptr(),
            1,
            gamma_abc_g1_ptr.as_mut_ptr(),
            1,
            res.as_mut_ptr(),
            1
        )
    };
    api_error::result_from(result).unwrap_or_revert();
    res
}