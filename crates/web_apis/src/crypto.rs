//! Web Crypto API implementation.

use boa_engine::{
    Context, JsArgs, JsResult, JsValue, NativeFunction,
    js_string,
    object::ObjectInitializer,
    property::Attribute,
};
use rand::RngCore;

/// Crypto API.
pub struct Crypto;

impl Crypto {
    /// Register the crypto API on the global object.
    pub fn register(context: &mut Context) {
        let subtle = ObjectInitializer::new(context)
            .function(NativeFunction::from_fn_ptr(subtle_encrypt), js_string!("encrypt"), 3)
            .function(NativeFunction::from_fn_ptr(subtle_decrypt), js_string!("decrypt"), 3)
            .function(NativeFunction::from_fn_ptr(subtle_sign), js_string!("sign"), 3)
            .function(NativeFunction::from_fn_ptr(subtle_verify), js_string!("verify"), 4)
            .function(NativeFunction::from_fn_ptr(subtle_digest), js_string!("digest"), 2)
            .function(NativeFunction::from_fn_ptr(subtle_generate_key), js_string!("generateKey"), 3)
            .function(NativeFunction::from_fn_ptr(subtle_derive_key), js_string!("deriveKey"), 5)
            .function(NativeFunction::from_fn_ptr(subtle_derive_bits), js_string!("deriveBits"), 3)
            .function(NativeFunction::from_fn_ptr(subtle_import_key), js_string!("importKey"), 5)
            .function(NativeFunction::from_fn_ptr(subtle_export_key), js_string!("exportKey"), 2)
            .function(NativeFunction::from_fn_ptr(subtle_wrap_key), js_string!("wrapKey"), 4)
            .function(NativeFunction::from_fn_ptr(subtle_unwrap_key), js_string!("unwrapKey"), 7)
            .build();

        let crypto = ObjectInitializer::new(context)
            .property(js_string!("subtle"), subtle, Attribute::READONLY)
            .function(NativeFunction::from_fn_ptr(crypto_get_random_values), js_string!("getRandomValues"), 1)
            .function(NativeFunction::from_fn_ptr(crypto_random_uuid), js_string!("randomUUID"), 0)
            .build();

        context
            .register_global_property(js_string!("crypto"), crypto, Attribute::all())
            .expect("Failed to register crypto");
    }

    /// Generate random bytes.
    pub fn get_random_values(buffer: &mut [u8]) {
        rand::thread_rng().fill_bytes(buffer);
    }

    /// Generate a random UUID.
    pub fn random_uuid() -> String {
        let mut bytes = [0u8; 16];
        rand::thread_rng().fill_bytes(&mut bytes);

        // Set version 4
        bytes[6] = (bytes[6] & 0x0f) | 0x40;
        // Set variant
        bytes[8] = (bytes[8] & 0x3f) | 0x80;

        format!(
            "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
            bytes[0], bytes[1], bytes[2], bytes[3],
            bytes[4], bytes[5],
            bytes[6], bytes[7],
            bytes[8], bytes[9],
            bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15]
        )
    }
}

// Native function implementations
fn crypto_get_random_values(_: &JsValue, args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    // Would fill the TypedArray with random values
    let array = args.get_or_undefined(0);
    Ok(array.clone())
}

fn crypto_random_uuid(_: &JsValue, _: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let uuid = Crypto::random_uuid();
    Ok(js_string!(uuid).into())
}

fn subtle_encrypt(_: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    use boa_engine::object::builtins::JsPromise;
    let (promise, _) = JsPromise::new_pending(context);
    Ok(promise.into())
}

fn subtle_decrypt(_: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    use boa_engine::object::builtins::JsPromise;
    let (promise, _) = JsPromise::new_pending(context);
    Ok(promise.into())
}

fn subtle_sign(_: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    use boa_engine::object::builtins::JsPromise;
    let (promise, _) = JsPromise::new_pending(context);
    Ok(promise.into())
}

fn subtle_verify(_: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    use boa_engine::object::builtins::JsPromise;
    let (promise, _) = JsPromise::new_pending(context);
    Ok(promise.into())
}

fn subtle_digest(_: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    use boa_engine::object::builtins::JsPromise;
    let (promise, _) = JsPromise::new_pending(context);
    Ok(promise.into())
}

fn subtle_generate_key(_: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    use boa_engine::object::builtins::JsPromise;
    let (promise, _) = JsPromise::new_pending(context);
    Ok(promise.into())
}

fn subtle_derive_key(_: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    use boa_engine::object::builtins::JsPromise;
    let (promise, _) = JsPromise::new_pending(context);
    Ok(promise.into())
}

fn subtle_derive_bits(_: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    use boa_engine::object::builtins::JsPromise;
    let (promise, _) = JsPromise::new_pending(context);
    Ok(promise.into())
}

fn subtle_import_key(_: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    use boa_engine::object::builtins::JsPromise;
    let (promise, _) = JsPromise::new_pending(context);
    Ok(promise.into())
}

fn subtle_export_key(_: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    use boa_engine::object::builtins::JsPromise;
    let (promise, _) = JsPromise::new_pending(context);
    Ok(promise.into())
}

fn subtle_wrap_key(_: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    use boa_engine::object::builtins::JsPromise;
    let (promise, _) = JsPromise::new_pending(context);
    Ok(promise.into())
}

fn subtle_unwrap_key(_: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    use boa_engine::object::builtins::JsPromise;
    let (promise, _) = JsPromise::new_pending(context);
    Ok(promise.into())
}

/// Supported algorithms.
#[derive(Clone, Debug)]
pub enum Algorithm {
    AesCbc { iv: Vec<u8> },
    AesCtr { counter: Vec<u8>, length: u8 },
    AesGcm { iv: Vec<u8>, additional_data: Option<Vec<u8>>, tag_length: u8 },
    RsaOaep { label: Option<Vec<u8>> },
    RsaPss { salt_length: u32 },
    Ecdsa { hash: HashAlgorithm },
    Hmac,
    Sha1,
    Sha256,
    Sha384,
    Sha512,
    Pbkdf2 { salt: Vec<u8>, iterations: u32, hash: HashAlgorithm },
    Hkdf { salt: Vec<u8>, info: Vec<u8>, hash: HashAlgorithm },
}

/// Hash algorithm.
#[derive(Clone, Copy, Debug)]
pub enum HashAlgorithm {
    Sha1,
    Sha256,
    Sha384,
    Sha512,
}

/// Key usage flags.
#[derive(Clone, Debug, Default)]
pub struct KeyUsages {
    pub encrypt: bool,
    pub decrypt: bool,
    pub sign: bool,
    pub verify: bool,
    pub derive_key: bool,
    pub derive_bits: bool,
    pub wrap_key: bool,
    pub unwrap_key: bool,
}

impl KeyUsages {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn encrypt(mut self) -> Self {
        self.encrypt = true;
        self
    }

    pub fn decrypt(mut self) -> Self {
        self.decrypt = true;
        self
    }

    pub fn sign(mut self) -> Self {
        self.sign = true;
        self
    }

    pub fn verify(mut self) -> Self {
        self.verify = true;
        self
    }
}

/// Crypto key.
#[derive(Clone, Debug)]
pub struct CryptoKey {
    /// Key type.
    pub key_type: KeyType,
    /// Whether the key is extractable.
    pub extractable: bool,
    /// Algorithm.
    pub algorithm: String,
    /// Key usages.
    pub usages: KeyUsages,
    /// Key data (internal).
    data: Vec<u8>,
}

/// Key type.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyType {
    Public,
    Private,
    Secret,
}

/// Key format.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyFormat {
    Raw,
    Pkcs8,
    Spki,
    Jwk,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_random_uuid() {
        let uuid = Crypto::random_uuid();
        assert_eq!(uuid.len(), 36);

        // Check format
        let parts: Vec<&str> = uuid.split('-').collect();
        assert_eq!(parts.len(), 5);
        assert_eq!(parts[0].len(), 8);
        assert_eq!(parts[1].len(), 4);
        assert_eq!(parts[2].len(), 4);
        assert_eq!(parts[3].len(), 4);
        assert_eq!(parts[4].len(), 12);

        // Check version (should be 4)
        assert!(parts[2].starts_with('4'));
    }

    #[test]
    fn test_get_random_values() {
        let mut buffer = [0u8; 32];
        Crypto::get_random_values(&mut buffer);

        // Very unlikely to be all zeros
        assert!(buffer.iter().any(|&b| b != 0));
    }

    #[test]
    fn test_key_usages() {
        let usages = KeyUsages::new().encrypt().decrypt().sign();
        assert!(usages.encrypt);
        assert!(usages.decrypt);
        assert!(usages.sign);
        assert!(!usages.verify);
    }
}
