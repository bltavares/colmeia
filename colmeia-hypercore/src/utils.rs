use ed25519_dalek::PublicKey;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum HashParserError {
    #[error("the bytes provided are not a valid hash")]
    InvalidHexHash(#[from] hex::FromHexError),
    #[error("the encoded hash is not a valid ed25519 public key")]
    InvalidPublicKey(#[from] ed25519_dalek::SignatureError),
}

pub trait PublicKeyExt {
    fn parse_from_hash(&self) -> Result<PublicKey, HashParserError>;
}

impl PublicKeyExt for [u8] {
    fn parse_from_hash(&self) -> Result<PublicKey, HashParserError> {
        let bytes = hex::decode(self)?;
        let key = PublicKey::from_bytes(&bytes)?;
        Ok(key)
    }
}

impl PublicKeyExt for &str {
    fn parse_from_hash(&self) -> Result<PublicKey, HashParserError> {
        self.as_bytes().parse_from_hash()
    }
}

impl PublicKeyExt for String {
    fn parse_from_hash(&self) -> Result<PublicKey, HashParserError> {
        self.as_bytes().parse_from_hash()
    }
}
