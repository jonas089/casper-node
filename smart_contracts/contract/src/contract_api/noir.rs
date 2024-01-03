use crate::{ext_ffi, unwrap_or_revert::UnwrapOrRevert};
use casper_types::api_error;
use alloc::{collections::BTreeSet, vec, vec::Vec, string::String};
extern crate std;
pub fn noir_verifier<T: AsRef<[u8]>>(proof: T) -> [u8;1]{
    // false
    let mut res: [u8;1] = [0;1];
    let result = unsafe {
        ext_ffi::casper_noir_verifier(
            proof.as_ref().as_ptr(),
            proof.as_ref().len(),
            res.as_mut_ptr(),
            1
        )
    };
    api_error::result_from(result).unwrap_or_revert();
    res
}