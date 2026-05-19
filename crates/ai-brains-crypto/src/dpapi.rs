use crate::errors::{CryptoError, Result};

#[cfg(windows)]
use windows::{
    core::PCWSTR,
    Win32::Security::Cryptography::{
        CryptProtectData, CryptUnprotectData, CRYPTPROTECT_UI_FORBIDDEN, CRYPT_INTEGER_BLOB,
    },
};

#[cfg(windows)]
pub fn wrap_key(key_material: &[u8]) -> Result<Vec<u8>> {
    let input = CRYPT_INTEGER_BLOB {
        cbData: key_material.len() as u32,
        pbData: key_material.as_ptr() as *mut u8,
    };

    let mut output = CRYPT_INTEGER_BLOB::default();

    unsafe {
        CryptProtectData(
            &input,
            PCWSTR::null(),
            None,
            None,
            None,
            CRYPTPROTECT_UI_FORBIDDEN,
            &mut output,
        )
        .map_err(|e| CryptoError::DpapiError(e.to_string()))?;
    }

    let result =
        unsafe { std::slice::from_raw_parts(output.pbData, output.cbData as usize).to_vec() };

    unsafe {
        windows::Win32::Foundation::LocalFree(windows::Win32::Foundation::HLOCAL(
            output.pbData as *mut core::ffi::c_void,
        ));
    }

    Ok(result)
}

#[cfg(windows)]
pub fn unwrap_key(wrapped_material: &[u8]) -> Result<Vec<u8>> {
    let input = CRYPT_INTEGER_BLOB {
        cbData: wrapped_material.len() as u32,
        pbData: wrapped_material.as_ptr() as *mut u8,
    };

    let mut output = CRYPT_INTEGER_BLOB::default();

    unsafe {
        CryptUnprotectData(
            &input,
            None,
            None,
            None,
            None,
            CRYPTPROTECT_UI_FORBIDDEN,
            &mut output,
        )
        .map_err(|e| CryptoError::DpapiError(e.to_string()))?;
    }

    let result =
        unsafe { std::slice::from_raw_parts(output.pbData, output.cbData as usize).to_vec() };

    unsafe {
        windows::Win32::Foundation::LocalFree(windows::Win32::Foundation::HLOCAL(
            output.pbData as *mut core::ffi::c_void,
        ));
    }

    Ok(result)
}

#[cfg(not(windows))]
pub fn wrap_key(_key_material: &[u8]) -> Result<Vec<u8>> {
    Err(CryptoError::DpapiError(
        "DPAPI is only available on Windows".to_string(),
    ))
}

#[cfg(not(windows))]
pub fn unwrap_key(_wrapped_material: &[u8]) -> Result<Vec<u8>> {
    Err(CryptoError::DpapiError(
        "DPAPI is only available on Windows".to_string(),
    ))
}

#[cfg(all(test, windows))]
mod tests {
    #![allow(clippy::disallowed_methods)]
    use super::*;

    #[test]
    fn windows_dpapi_roundtrip() {
        let key = b"this is a secret key 123456789012";
        let wrapped = wrap_key(key).expect("Failed to wrap");
        assert_ne!(key.to_vec(), wrapped);

        let unwrapped = unwrap_key(&wrapped).expect("Failed to unwrap");
        assert_eq!(key.to_vec(), unwrapped);
    }
}
