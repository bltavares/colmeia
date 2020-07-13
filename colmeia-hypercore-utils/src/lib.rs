use blake2_rfc::blake2b::Blake2b;
use ed25519_dalek::PublicKey;
// TODO make it a Hypercore URL
use parse_dat_url::DatUrl;

pub use parse_dat_url::Error;

const HYPERCORE: [u8; 9] = *b"hypercore";

const HOSTNAME: &str = ".hyperswarm.local.";

fn discovery_key(public_key: &PublicKey) -> Vec<u8> {
    let mut hasher = Blake2b::with_key(32, public_key.as_bytes());
    hasher.update(&HYPERCORE);
    hasher.finalize().as_bytes().into()
}

fn dns_discovery_key(discovery_key: &[u8]) -> String {
    hex::encode(discovery_key).chars().take(40).collect()
}

#[derive(Debug, Clone)]
pub struct HashUrl {
    public_key: PublicKey,
    discovery_key: Vec<u8>,
    local_dns_domain: String,
    original: DatUrl<'static>,
}

#[derive(Clone)]
pub enum UrlResolution<'a> {
    HashUrl(HashUrl),
    RemoteUrl(DatUrl<'a>),
}

/// Parses a url as a Dat based url
/// Supports a hash or a regular DNS domain
///
/// # Errors
///
/// If the string cannot be parsed as a regular DNS domain or Dat Public Key
pub fn parse(dat_name: &str) -> Result<UrlResolution, Error> {
    let dat_url = DatUrl::parse(dat_name)?.into_owned();
    let public_key_bytes = hex::decode(dat_url.host().as_bytes());

    let public_key_bytes = match public_key_bytes {
        Ok(result) => result,
        Err(_) => return Ok(UrlResolution::RemoteUrl(dat_url)),
    };

    let public_key = PublicKey::from_bytes(&public_key_bytes);
    let public_key = match public_key {
        Ok(result) => result,
        Err(_) => return Ok(UrlResolution::RemoteUrl(dat_url)),
    };

    Ok(UrlResolution::HashUrl(HashUrl::new(public_key, dat_url)))
}

impl HashUrl {
    fn new(public_key: PublicKey, original: DatUrl<'static>) -> Self {
        let discovery_key = discovery_key(&public_key);
        let local_dns_domain = dns_discovery_key(&discovery_key) + HOSTNAME;

        HashUrl {
            public_key,
            local_dns_domain,
            discovery_key,
            original,
        }
    }

    #[must_use]
    pub fn local_dns_domain(&self) -> &str {
        &self.local_dns_domain
    }

    #[must_use]
    pub fn discovery_key(&self) -> &[u8] {
        &self.discovery_key
    }

    #[must_use]
    pub fn public_key(&self) -> &PublicKey {
        &self.public_key
    }
}
