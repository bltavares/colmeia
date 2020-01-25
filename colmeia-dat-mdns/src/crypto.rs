use blake2_rfc::blake2b::Blake2b;
use ed25519_dalek::PublicKey;

const HYPERCORE: [u8; 9] = *b"hypercore";

const HOSTNAME: &str = ".dat.local.";

fn to_discovery_key(public_key: PublicKey) -> Vec<u8> {
  let mut hasher = Blake2b::with_key(32, public_key.as_bytes());
  hasher.update(&HYPERCORE);
  hasher.finalize().as_bytes().into()
}

fn to_dns_discovery_key(discovery_key: &[u8]) -> String {
  hex::encode(discovery_key).chars().take(40).collect()
}

#[derive(Clone)]
pub struct DatLocalDiscoverUrl(pub(crate) String);

// TODO: Create a error type
pub fn dat_url_mdns_discovery_name(
  dat_name: &str,
) -> Result<DatLocalDiscoverUrl, parse_dat_url::Error> {
  let dat_url = parse_dat_url::DatUrl::parse(dat_name)?;
  let public_key_bytes = hex::decode(dat_url.host().as_bytes()).expect("valid public key");
  let public_key = PublicKey::from_bytes(&public_key_bytes).expect("invalid key");
  let url = to_dns_discovery_key(&to_discovery_key(public_key)) + HOSTNAME;
  Ok(DatLocalDiscoverUrl(url))
}
