use crate::{ext_ffi, unwrap_or_revert::UnwrapOrRevert};
use casper_types::api_error;
use alloc::{collections::BTreeSet, vec, vec::Vec, string::String};

pub fn circom_verifier<T: AsRef<[u8]>>(inputs: T) -> Vec<u8>{
    let mut res: Vec<u8> = Vec::new();
    let result = unsafe {
        ext_ffi::casper_circom_verifier(
            inputs.as_ref().as_ptr(),
            1,
            res.as_mut_ptr(),
            1
        )
    };
    api_error::result_from(result).unwrap_or_revert();
    res
}