use crate::{ext_ffi, unwrap_or_revert::UnwrapOrRevert};
use casper_types::api_error;

pub fn miden_verifier() -> [u8;1]{
    let mut ret = [0; 1];
    let result = unsafe {
        ext_ffi::miden_verifier(
            ret.as_mut_ptr(),
            1,
        )
    };
    api_error::result_from(result).unwrap_or_revert();
    ret
}