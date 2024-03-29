use hmac::{Mac, Hmac};

use crate::net;

// for the context below, would it be better to have this specified by the 
// environment since this is technically an open source project

/// required for the blake3 key derivation
pub const BLAKE3_CONTEXT: &str = "thoughts_server 20221031 security::mac";

#[derive(Debug)]
pub enum Error {
    InvalidKeyLength
}

pub type Result<T> = std::result::Result<T, Error>;

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidKeyLength => write!(f, "given key is an invalid length")
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

impl From<hmac::digest::InvalidLength> for Error {
    fn from(_: hmac::digest::InvalidLength) -> Self {
        Error::InvalidKeyLength
    }
}

impl From<Error> for net::http::error::Error {
    fn from(err: Error) -> Self {
        net::http::error::Error::new()
            .set_source(err)
    }
}

macro_rules! hmac_methods {
    ($make:ident, $once:ident, $verify:ident, $e:path) => {
        /// create a new hmac
        fn $make(secret: &[u8], data: &[u8])-> Result<Hmac<$e>> {
            let mut mac = Hmac::new_from_slice(secret)?;
            mac.update(data);
            Ok(mac)
        }

        /// a one off hmac
        pub fn $once<S,D>(secret: S, data: D) -> Result<Vec<u8>>
        where
            S: AsRef<[u8]>,
            D: AsRef<[u8]>,
        {
            let result = $make(secret.as_ref(), data.as_ref())?.finalize();
            let bytes = result.into_bytes();
            Ok(bytes.to_vec())
        }

        /// verify a given hmac
        pub fn $verify<S,D,M>(secret: S, data: D, mac: M) -> Result<bool>
        where
            S: AsRef<[u8]>,
            D: AsRef<[u8]>,
            M: AsRef<[u8]>
        {
            let result = $make(secret.as_ref(), data.as_ref())?;

            Ok(match result.verify_slice(mac.as_ref()) {
                Ok(()) => true,
                Err(_e) => false
            })
        }
    };
}

hmac_methods!(make_sha224, one_off_sha224, one_off_verify_sha224, sha3::Sha3_224);
hmac_methods!(make_sha256, one_off_sha256, one_off_verify_sha256, sha3::Sha3_256);
hmac_methods!(make_sha384, one_off_sha384, one_off_verify_sha384, sha3::Sha3_384);
hmac_methods!(make_sha512, one_off_sha512, one_off_verify_sha512, sha3::Sha3_512);

/// create [blake3::Hash] from given secret and data
/// 
/// it will use the [BLAKE3_CONTEXT] for key derivation to be used for the
/// hasher and then return the finalized hash
fn make_blake3(secret: &[u8], data: &[u8]) -> blake3::Hash {
    let key = blake3::derive_key(BLAKE3_CONTEXT, secret);
    let mut hasher = blake3::Hasher::new_keyed(&key);
    blake3::Hasher::update(&mut hasher, data);
    blake3::Hasher::finalize(&hasher)
}

/// creates a mac via [blake3::Hash]
pub fn one_off_blake3<S,D>(secret: S, data: D) -> Result<Vec<u8>>
where
    S: AsRef<[u8]>,
    D: AsRef<[u8]>,
{
    let hash = make_blake3(secret.as_ref(), data.as_ref());
    let bytes = hash.as_bytes();
    let mut rtn = Vec::with_capacity(bytes.len());

    for b in bytes {
        rtn.push(b.clone());
    }

    Ok(rtn)
}

/// validates secret and data against an existing mac using [blake3::Hash]
pub fn one_off_verify_blake3<S,D,M>(secret: S, data: D, mac: M) -> Result<bool>
where
    S: AsRef<[u8]>,
    D: AsRef<[u8]>,
    M: AsRef<[u8]>,
{
    let mac_ref = mac.as_ref();

    if mac_ref.len() != blake3::OUT_LEN {
        return Err(Error::InvalidKeyLength);
    }

    let hash = make_blake3(secret.as_ref(), data.as_ref());
    let cmp = {
        // not sure if this is optimal or if something else should be done
        // since this will get called a lot
        let mut bytes = [0u8; 32];

        for index in 0..mac_ref.len() {
            bytes[index] = mac_ref[index];
        }

        blake3::Hash::from(bytes)
    };

    Ok(hash == cmp)
}

#[allow(non_camel_case_types)]
pub enum Algorithm {
    HMAC_SHA224,
    HMAC_SHA256,
    HMAC_SHA384,
    HMAC_SHA512,
    BLAKE3
}

/// runs a one_off using algorithm
pub fn algo_one_off<S,D>(algo: &Algorithm, secret: S, data: D) -> Result<Vec<u8>>
where
    S: AsRef<[u8]>,
    D: AsRef<[u8]>,
{
    match algo {
        Algorithm::HMAC_SHA224 => one_off_sha224(secret, data),
        Algorithm::HMAC_SHA256 => one_off_sha256(secret, data),
        Algorithm::HMAC_SHA384 => one_off_sha384(secret, data),
        Algorithm::HMAC_SHA512 => one_off_sha512(secret, data),
        Algorithm::BLAKE3 => one_off_blake3(secret, data)
    }
}

/// runs a one_off_verify using algorithm
pub fn algo_one_off_verify<S,D,M>(algo: &Algorithm, secret: S, data: D, mac: M) -> Result<bool>
where
    S: AsRef<[u8]>,
    D: AsRef<[u8]>,
    M: AsRef<[u8]>
{
    match algo {
        Algorithm::HMAC_SHA224 => one_off_verify_sha224(secret, data, mac),
        Algorithm::HMAC_SHA256 => one_off_verify_sha256(secret, data, mac),
        Algorithm::HMAC_SHA384 => one_off_verify_sha384(secret, data, mac),
        Algorithm::HMAC_SHA512 => one_off_verify_sha512(secret, data, mac),
        Algorithm::BLAKE3 => one_off_verify_blake3(secret, data, mac)
    }
}
