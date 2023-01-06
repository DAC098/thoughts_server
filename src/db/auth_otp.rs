use std::convert::TryFrom;

use tokio_postgres::GenericClient;

use super::error;

/// possible algorithm values for totp
#[derive(Clone)]
pub enum Algo {
    SHA1,
    SHA256,
    SHA512
}

impl Algo {

    pub fn try_from_i16(v: i16) -> std::result::Result<Algo, ()> {
        match v {
            0 => Ok(Algo::SHA1),
            1 => Ok(Algo::SHA256),
            2 => Ok(Algo::SHA512),
            _ => Err(())
        }
    }

    pub fn from_i16(v: i16) -> Algo {
        Self::try_from_i16(v).expect("unexpected value for Algo")
    }

    pub fn into_i16(self) -> i16 {
        match self {
            Algo::SHA1 => 0,
            Algo::SHA256 => 1,
            Algo::SHA512 => 2
        }
    }

    pub fn try_from_str<S>(v: S) -> std::result::Result<Algo, ()>
    where
        S: AsRef<str>
    {
        match v.as_ref() {
            "SHA1" => Ok(Algo::SHA1),
            "SHA256" => Ok(Algo::SHA256),
            "SHA512" => Ok(Algo::SHA512),
            _ => Err(())
        }
    }

    pub fn into_string(self) -> String {
        match self {
            Algo::SHA1 => "SHA1",
            Algo::SHA256 => "SHA256",
            Algo::SHA512 => "SHA512"
        }.to_owned()
    }
}

impl TryFrom<&str> for Algo
{
    type Error = ();

    #[inline]
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_from_str(value)
    }
}

impl TryFrom<String> for Algo
{
    type Error = ();

    #[inline]
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from_str(value)
    }
}

impl TryFrom<i16> for Algo
{
    type Error = ();

    #[inline]
    fn try_from(value: i16) -> Result<Self, Self::Error> {
        Self::try_from_i16(value)
    }
}

impl Into<i16> for Algo
{
    #[inline]
    fn into(self) -> i16 {
        self.into_i16()
    }
}

impl Into<String> for Algo
{
    #[inline]
    fn into(self) -> String {
        self.into_string()
    }
}

/// record format for storing totp information in the database
pub struct AuthOtp {
    pub id: i32,
    pub users_id: i32,
    pub algo: Algo,
    pub secret: Vec<u8>,
    pub digits: i16,
    pub step: i16,
    pub verified: bool
}

impl AuthOtp {

    /// find totp record for specific user
    /// 
    /// a user should only have one record so this will only return one record
    /// if it was found
    pub async fn find_users_id(conn: &impl GenericClient, users_id: &i32) -> error::Result<Option<AuthOtp>> {
        if let Some(row) = conn.query_opt(
            "\
            select id, \
                   users_id, \
                   algo, \
                   secret, \
                   digits, \
                   step, \
                   verified \
            from auth_otp \
            where users_id = $1",
            &[users_id]
        ).await? {
            Ok(Some(AuthOtp {
                id: row.get(0),
                users_id: row.get(1),
                algo: Algo::from_i16(row.get(2)),
                secret: row.get(3),
                digits: row.get(4),
                step: row.get(5),
                verified: row.get(6)
            }))
        } else {
            Ok(None)
        }
    }
}