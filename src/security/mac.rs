#[derive(Debug)]
pub enum Error {
    InvalidKeyLength
}

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

// for the context below, would it be better to have this specified by the 
// environment since this is technically an open source project

/// required for the blake3 key derivation
pub const BLAKE3_CONTEXT: &str = "thoughts_server 20221031 security::mac";

/*
// this is an attempt to allow for different options when doing mac's. I dont
// really want to fuck with all the different generics for the various types
// so I will just use BLAKE3 until I figure out how to do it

pub enum HmacAlgorithms {
    SHA2_224,
    SHA2_256,
    SHA2_384,
    SHA2_512,

    SHA3_224,
    SHA3_256,
    SHA3_384,
    SHA3_512,
}

pub enum MacAlgorithms {
    HMAC(HmacAlgorithms),
    BLAKE3
}

pub fn hmac_one_off(alg: HmacAlgorithms, secret: &[u8], data: &[u8]) -> std::result::Result<(), Error>
{
    Ok(match alg {
        HmacAlgorithms::SHA2_224 => {
            let mut hash = Hmac::<sha2::Sha224>::new_from_slice(secret)?;
            hash.update(data);
            hash.finalize().into_bytes()
        },
        HmacAlgorithms::SHA2_256 => {
            let mut hash = Hmac::<sha2::Sha256>::new_from_slice(secret)?;
            hash.update(data);
            hash.finalize().into_bytes()
        },
        HmacAlgorithms::SHA2_384 => {
            let mut hash = Hmac::<sha2::Sha384>::new_from_slice(secret)?;
            hash.update(data);
            hash.finalize().into_bytes()
        },
        HmacAlgorithms::SHA2_512 => {
            let mut hash = Hmac::<sha2::Sha512>::new_from_slice(secret)?;
            hash.update(data);
            hash.finalize().into_bytes()
        },

        HmacAlgorithms::SHA3_224 => {
            let mut hash = Hmac::<sha3::Sha3_224>::new_from_slice(secret)?;
            hash.update(data);
            hash.finalize().into_bytes()
        },
        HmacAlgorithms::SHA3_256 => {
            let mut hash = Hmac::<sha3::Sha3_256>::new_from_slice(secret)?;
            hash.update(data);
            hash.finalize().into_bytes()
        },
        HmacAlgorithms::SHA3_384 => {
            let mut hash = Hmac::<sha3::Sha3_384>::new_from_slice(secret)?;
            hash.update(data);
            hash.finalize().into_bytes()
        },
        HmacAlgorithms::SHA3_512 => {
            let mut hash = Hmac::<sha3::Sha3_512>::new_from_slice(secret)?;
            hash.update(data);
            hash.finalize().into_bytes()
        }
    })
}
*/

/// create [`blake3::Hash`] from given secret and data
/// 
/// it will use the [`BLAKE3_CONTEXT`] for key derivation to be used for the
/// hasher and then return the finalized hash
fn make_hash(secret: &[u8], data: &[u8]) -> blake3::Hash {
    let key = blake3::derive_key(BLAKE3_CONTEXT, secret);
    let mut hasher = blake3::Hasher::new_keyed(&key);
    hasher.update(data);
    hasher.finalize()
}

/// creates a mac via BLAKE3 hashing
pub fn one_off(secret: &[u8], data: &[u8]) -> Vec<u8> {
    let hash = make_hash(secret, data);
    let bytes = hash.as_bytes();
    let mut rtn = Vec::with_capacity(bytes.len());

    for b in bytes {
        rtn.push(b.clone());
    }

    rtn
}

/// result returned from [`one_off_verify`]
pub enum VerifyResult {
    /// the mac and data given matched
    Valid,
    /// the mac data data given did not match
    Invalid,
    /// the mac provided was too long
    InvalidLength,
}

/// validates secret and data against an existing mac using BLAKE3 hashing
pub fn one_off_verify(secret: &[u8], data: &[u8], mac: &[u8]) -> VerifyResult {
    if mac.len() != blake3::OUT_LEN {
        return VerifyResult::InvalidLength;
    }

    let hash = make_hash(secret, data);
    let cmp = {
        // not sure if this is optimal or if something else should be done
        // since this will get called a lot
        let mut bytes = [0u8; 32];

        for index in 0..mac.len() {
            bytes[index] = mac[index];
        }

        blake3::Hash::from(bytes)
    };

    if hash == cmp {
        VerifyResult::Valid
    } else {
        VerifyResult::Invalid
    }
}