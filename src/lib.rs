extern crate base64;
extern crate challenge_bypass_ristretto;
extern crate core;
extern crate rand;
extern crate sha2;

use core::ptr;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

use challenge_bypass_ristretto::{
    BlindedToken, SignedToken, SigningKey, Token, TokenPreimage, UnblindedToken, VerificationKey,
    VerificationSignature,
};
use rand::rngs::OsRng;
use sha2::Sha512;

/// Destroy a `*c_char` once you are done with it.
#[no_mangle]
pub unsafe extern "C" fn c_char_destroy(s: *mut c_char) {
    if !s.is_null() {
        drop(CString::from_raw(s));
    }
}

macro_rules! impl_base64 {
    ($t:ident, $en:ident, $de:ident) => {
        /// Return base64 encoding as a C string.
        #[no_mangle]
        pub unsafe extern "C" fn $en(t: *const $t) -> *mut c_char {
            if !t.is_null() {
                let b64 = (&*t).encode_base64();
                if let Ok(s) = CString::new(b64) {
                    return s.into_raw();
                }
            }
            return ptr::null_mut();
        }

        /// Decode base64 C string.
        ///
        /// If something goes wrong, this will return a null pointer. Don't forget to
        /// destroy the returned pointer once you are done with it!
        #[no_mangle]
        pub unsafe extern "C" fn $de(s: *const c_char) -> *mut $t {
            if !s.is_null() {
                let raw = CStr::from_ptr(s);
                if let Ok(s_as_str) = raw.to_str() {
                    if let Ok(t) = $t::decode_base64(s_as_str) {
                        return Box::into_raw(Box::new(t));
                    }
                }
            }
            return ptr::null_mut();
        }
    };
}

/// Destroy a `TokenPreimage` once you are done with it.
#[no_mangle]
pub unsafe extern "C" fn token_preimage_destroy(t: *mut TokenPreimage) {
    if !t.is_null() {
        drop(Box::from_raw(t));
    }
}

impl_base64!(
    TokenPreimage,
    token_preimage_encode_base64,
    token_preimage_decode_base64
);

/// Generate a new `Token`
///
/// # Safety
///
/// Make sure you destroy the token with [`token_destroy()`] once you are
/// done with it.
#[no_mangle]
pub unsafe extern "C" fn token_random() -> *mut Token {
    let mut rng = OsRng::new().unwrap();
    let token = Token::random(&mut rng);
    Box::into_raw(Box::new(token))
}

/// Destroy a `Token` once you are done with it.
#[no_mangle]
pub unsafe extern "C" fn token_destroy(token: *mut Token) {
    if !token.is_null() {
        drop(Box::from_raw(token));
    }
}

/// Take a reference to a `Token` and blind it, returning a `BlindedToken`
///
/// If something goes wrong, this will return a null pointer. Don't forget to
/// destroy the `BlindedToken` once you are done with it!
#[no_mangle]
pub unsafe extern "C" fn token_blind(token: *const Token) -> *mut BlindedToken {
    if token.is_null() {
        return ptr::null_mut();
    }

    Box::into_raw(Box::new((*token).blind()))
}

/// Take a reference to a `Token` and use it to unblind a `SignedToken`, returning an `UnblindedToken`
///
/// If something goes wrong, this will return a null pointer. Don't forget to
/// destroy the `UnblindedToken` once you are done with it!
#[no_mangle]
pub unsafe extern "C" fn token_unblind(
    token: *const Token,
    signed_token: *const SignedToken,
) -> *mut UnblindedToken {
    if token.is_null() {
        return ptr::null_mut();
    }
    if signed_token.is_null() {
        return ptr::null_mut();
    }
    match (*token).unblind(&*signed_token) {
        Ok(unblinded_token) => Box::into_raw(Box::new(unblinded_token)),
        Err(_) => ptr::null_mut(),
    }
}

impl_base64!(Token, token_encode_base64, token_decode_base64);

/// Destroy a `BlindedToken` once you are done with it.
#[no_mangle]
pub unsafe extern "C" fn blinded_token_destroy(token: *mut BlindedToken) {
    if !token.is_null() {
        drop(Box::from_raw(token));
    }
}

impl_base64!(
    BlindedToken,
    blinded_token_encode_base64,
    blinded_token_decode_base64
);

/// Destroy a `SignedToken` once you are done with it.
#[no_mangle]
pub unsafe extern "C" fn signed_token_destroy(token: *mut SignedToken) {
    if !token.is_null() {
        drop(Box::from_raw(token));
    }
}

impl_base64!(
    SignedToken,
    signed_token_encode_base64,
    signed_token_decode_base64
);

/// Destroy an `UnblindedToken` once you are done with it.
#[no_mangle]
pub unsafe extern "C" fn unblinded_token_destroy(token: *mut UnblindedToken) {
    if !token.is_null() {
        drop(Box::from_raw(token));
    }
}

/// Take a reference to an `UnblindedToken` and use it to derive a `VerificationKey`
/// using Sha512 as the hash function
///
/// If something goes wrong, this will return a null pointer. Don't forget to
/// destroy the `VerificationKey` once you are done with it!
#[no_mangle]
pub unsafe extern "C" fn unblinded_token_derive_verification_key_sha512(
    token: *const UnblindedToken,
) -> *mut VerificationKey {
    if token.is_null() {
        return ptr::null_mut();
    }
    Box::into_raw(Box::new((*token).derive_verification_key::<Sha512>()))
}

/// Take a reference to an `UnblindedToken` and return the corresponding `TokenPreimage`
///
/// If something goes wrong, this will return a null pointer. Don't forget to
/// destroy the `BlindedToken` once you are done with it!
#[no_mangle]
pub unsafe extern "C" fn unblinded_token_preimage(
    token: *const UnblindedToken,
) -> *mut TokenPreimage {
    if token.is_null() {
        return ptr::null_mut();
    }

    Box::into_raw(Box::new((*token).t))
}

impl_base64!(
    UnblindedToken,
    unblinded_token_encode_base64,
    unblinded_token_decode_base64
);

/// Destroy a `VerificationKey` once you are done with it.
#[no_mangle]
pub unsafe extern "C" fn verification_key_destroy(key: *mut VerificationKey) {
    if !key.is_null() {
        drop(Box::from_raw(key));
    }
}

/// Take a reference to a `VerificationKey` and use it to sign a message
/// using Sha512 as the HMAC hash function to obtain a `VerificationSignature`
///
/// If something goes wrong, this will return a null pointer. Don't forget to
/// destroy the `VerificationSignature` once you are done with it!
#[no_mangle]
pub unsafe extern "C" fn verification_key_sign_sha512(
    key: *const VerificationKey,
    message: *const c_char,
) -> *mut VerificationSignature {
    if key.is_null() {
        return ptr::null_mut();
    }

    let raw = CStr::from_ptr(message);

    let message_as_str = match raw.to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };
    Box::into_raw(Box::new((*key).sign::<Sha512>(message_as_str.as_bytes())))
}

/// Take a reference to a `VerificationKey` and use it to verify an
/// existing `VerificationSignature` using Sha512 as the HMAC hash function
///
#[no_mangle]
pub unsafe extern "C" fn verification_key_verify_sha512(
    key: *const VerificationKey,
    sig: *const VerificationSignature,
    message: *const c_char,
) -> bool {
    if key.is_null() || sig.is_null() {
        return false;
    }

    let raw = CStr::from_ptr(message);

    let message_as_str = match raw.to_str() {
        Ok(s) => s,
        Err(_) => return false,
    };
    (*key).verify::<Sha512>(&*sig, message_as_str.as_bytes())
}

/// Destroy a `VerificationSignature` once you are done with it.
#[no_mangle]
pub unsafe extern "C" fn verification_signature_destroy(sig: *mut VerificationSignature) {
    if !sig.is_null() {
        drop(Box::from_raw(sig));
    }
}

impl_base64!(
    VerificationSignature,
    verification_signature_encode_base64,
    verification_signature_decode_base64
);

/// Generate a new `SigningKey`
///
/// # Safety
///
/// Make sure you destroy the key with [`signing_key_destroy()`] once you are
/// done with it.
#[no_mangle]
pub unsafe extern "C" fn signing_key_random() -> *mut SigningKey {
    let mut rng = OsRng::new().unwrap();
    let key = SigningKey::random(&mut rng);
    Box::into_raw(Box::new(key))
}

/// Destroy a `SigningKey` once you are done with it.
#[no_mangle]
pub unsafe extern "C" fn signing_key_destroy(key: *mut SigningKey) {
    if !key.is_null() {
        drop(Box::from_raw(key));
    }
}

/// Take a reference to a `SigningKey` and use it to sign a `BlindedToken`, returning a
/// `SignedToken`
///
/// If something goes wrong, this will return a null pointer. Don't forget to
/// destroy the `SignedToken` once you are done with it!
#[no_mangle]
pub unsafe extern "C" fn signing_key_sign(
    key: *const SigningKey,
    token: *const BlindedToken,
) -> *mut SignedToken {
    if key.is_null() {
        return ptr::null_mut();
    }

    if token.is_null() {
        return ptr::null_mut();
    }

    match (*key).sign(&*token) {
        Ok(signed_token) => Box::into_raw(Box::new(signed_token)),
        Err(_) => ptr::null_mut(),
    }
}

/// Take a reference to a `SigningKey` and use it to rederive an `UnblindedToken`
///
/// If something goes wrong, this will return a null pointer. Don't forget to
/// destroy the `UnblindedToken` once you are done with it!
#[no_mangle]
pub unsafe extern "C" fn signing_key_rederive_unblinded_token(
    key: *const SigningKey,
    t: *const TokenPreimage,
) -> *mut UnblindedToken {
    if key.is_null() {
        return ptr::null_mut();
    }

    if t.is_null() {
        return ptr::null_mut();
    }

    Box::into_raw(Box::new((*key).rederive_unblinded_token(&*t)))
}

impl_base64!(
    SigningKey,
    signing_key_encode_base64,
    signing_key_decode_base64
);
