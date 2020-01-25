use blake2_rfc::blake2b::Blake2b;
use ed25519_dalek::PublicKey;
use parse_dat_url::DatUrl;

pub use parse_dat_url::Error;

const HYPERCORE: [u8; 9] = *b"hypercore";

const HOSTNAME: &str = ".dat.local.";

fn discovery_key(public_key: &PublicKey) -> Vec<u8> {
    let mut hasher = Blake2b::with_key(32, public_key.as_bytes());
    hasher.update(&HYPERCORE);
    hasher.finalize().as_bytes().into()
}

fn dns_discovery_key(discovery_key: &[u8]) -> String {
    hex::encode(discovery_key).chars().take(40).collect()
}

#[derive(Debug, Clone)]
pub struct HashUrl<'a> {
    public_key: PublicKey,
    discovery_key: Vec<u8>,
    local_dns_domain: String,
    original: DatUrl<'a>,
}

pub enum DatUrlResolution<'a> {
    HashUrl(HashUrl<'a>),
    RemoteUrl(DatUrl<'a>),
}

pub fn parse<'a>(dat_name: &'a str) -> Result<DatUrlResolution<'a>, Error> {
    let dat_url = DatUrl::parse(dat_name)?;
    let public_key_bytes = hex::decode(dat_url.host().as_bytes());

    let public_key_bytes = match public_key_bytes {
        Ok(result) => result,
        Err(_) => return Ok(DatUrlResolution::RemoteUrl(dat_url)),
    };

    let public_key = PublicKey::from_bytes(&public_key_bytes);
    let public_key = match public_key {
        Ok(result) => result,
        Err(_) => return Ok(DatUrlResolution::RemoteUrl(dat_url)),
    };

    Ok(DatUrlResolution::HashUrl(HashUrl::new(public_key, dat_url)))
}

impl<'a> HashUrl<'a> {
    fn new(public_key: PublicKey, original: DatUrl<'a>) -> Self {
        let discovery_key = discovery_key(&public_key);
        let local_dns_domain = dns_discovery_key(&discovery_key) + HOSTNAME;

        HashUrl {
            public_key,
            local_dns_domain,
            discovery_key,
            original,
        }
    }

    pub fn local_dns_domain(&self) -> &str {
        &self.local_dns_domain
    }

    pub fn discovery_key(&self) -> &[u8] {
        &self.discovery_key
    }

    pub fn public_key(&self) -> &PublicKey {
        &self.public_key
    }
}
