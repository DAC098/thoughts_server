use super::mac;

/// default step for totp
pub const DEFAULT_STEP: u64 = 30;
/// default digit legnth for totp
pub const DEFAULT_DIGITS: u32 = 8;

/// the available algorithms for otp
pub enum Algorithm {
    SHA1,
    SHA256,
    SHA512
}

fn one_off(algo: Algorithm, secret: &[u8], data: &[u8]) -> mac::Result<Vec<u8>> {
    match algo {
        Algorithm::SHA1 => mac::one_off_sha1(secret, data),
        Algorithm::SHA256 => mac::one_off_sha256(secret, data),
        Algorithm::SHA512 => mac::one_off_sha512(secret, data)
    }
}

/// simple string padding given a string and total digits
/// 
/// this will not truncate the string and will just return if the given string
/// is big enough or is equal to the given digits
fn pad_string(uint_string: String, digits: usize) -> String {
    if uint_string.len() < digits {
        let mut rtn = String::with_capacity(digits);

        for _ in 0..(digits - uint_string.len()) {
            rtn.push('0');
        }

        rtn.push_str(&uint_string);
        rtn
    } else {
        uint_string
    }
}

/// generate integer string for otp algorithms
/// 
/// creates the integer string for the given algorithm. will pad the string
/// if it is not long enough for the given amount of digits.
fn generate_integer_string(algorithm: Algorithm, secret: &[u8], digits: u32, data: &[u8]) -> String {
    let hash = one_off(algorithm, secret, data).unwrap();

    let offset = (hash[hash.len() - 1] & 0xf) as usize;
    let binary = 
        ((hash[offset] & 0x7f) as u64) << 24 |
        (hash[offset + 1] as u64) << 16 |
        (hash[offset + 2] as u64) <<  8 |
        (hash[offset + 3] as u64);

    let uint_string = (binary % 10u64.pow(digits)).to_string();
    let digits = digits as usize;

    pad_string(uint_string, digits)
}

/// create an hotp hash
pub fn hotp<S>(secret: S, digits: u32, counter: u64) -> String
where
    S: AsRef<[u8]>
{
    let counter_bytes = counter.to_be_bytes();
    
    generate_integer_string(Algorithm::SHA1, secret.as_ref(), digits, &counter_bytes)
}

// create an totp hash
pub fn totp<S>(algorithm: Algorithm, secret: S, digits: u32, step: u64, time: u64) -> String
where
    S: AsRef<[u8]>
{
    let data = (time / step).to_be_bytes();

    generate_integer_string(algorithm, secret.as_ref(), digits, &data)
}