use std::env;

use colmeia_hyperstack::utils::{HashParserError, PublicKeyExt};
use ed25519_dalek::PublicKey;

fn main() {
    let hyper_hash = env::args()
        .nth(1)
        .expect("provide a hyper hash as argument");

    let key = hyper_hash.parse_from_hash();

    match key {
        Ok(public) => {
            valid_public_key(&public);
        }
        Err(e) => {
            invalid_public_key(e);
        }
    }
}

fn invalid_public_key(e: HashParserError) -> () {
    match e {
        HashParserError::InvalidHexHash(_) => {
            eprintln!("Invalid hyper url");
        }
        HashParserError::InvalidPublicKey(_) => {
            eprintln!("Invalid public key. This is likely the discovery key.");
        }
    }
}

fn valid_public_key(public: &PublicKey) {
    println!("Valid public key.");
    let bytes = public.as_bytes();
    println!("Public Hexa\t{}", hex::encode(bytes));
    println!("Public Bytes\t{:?}", bytes);

    let topic = hypercore_protocol::discovery_key(bytes);

    println!("Discovery Hexa\t{}", hex::encode(&topic));
    println!("Discovery Bytes\t{:?}", topic);
}
