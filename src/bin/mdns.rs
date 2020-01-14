use blake2_rfc::blake2b::Blake2b;
use ed25519_dalek::PublicKey;

const HOSTNAME: &str = ".dat.local";
const HYPERCORE: [u8; 9] = *b"hypercore";

type Error = Result<(), parse_dat_url::Error>;

fn to_discovery_key(public_key: PublicKey) -> Vec<u8> {
  let mut hasher = Blake2b::with_key(32, public_key.as_bytes());
  hasher.update(&HYPERCORE);
  hasher.finalize().as_bytes().into()
}

fn to_dns_discovery_key(discovery_key: &[u8]) -> String {
  hex::encode(discovery_key).chars().take(40).collect()
}

/// Possible to find TXT replies from running DAT on the same computer
/// But given how the protocol works, there is no usable IP
/// The IP origin of the response is not returned.
/// It also returns a lot of other records that are not part of the query (such as chromecast. likely a bug on chromecast itself?)
///
/// [src/bin/mdns.rs:33] response = Ok(
// Response {
//   answers: [
//       Record {
//           name: "2e020eeb63d2767211d2fa5ba97aa8a46b1b5577.dat.local",
//           class: IN,
//           ttl: 0,
//           kind: TXT(
//               [
//                   "token=clbcGFm+unK8+RgSq6TiQNPAagotxuNdqJoaHX1zMWU=",
//                   "peers=AAAAAAzS",
//               ],
//           ),
//       },
//   ],
//   nameservers: [],
//   additional: [],
// },
// )
fn mdns_lib(url: &str) {
  for response in mdns::discover::all(url).unwrap() {
    let response = dbg!(response).unwrap();
    for record in response.records() {
      dbg!(record);
    }
  }
}

/// Gets stuck on sending the query
///
// [DEBUG trust_dns_resolver::async_resolver] trust-dns resolver running
// [src/bin/mdns.rs:65] &url = "2e020eeb63d2767211d2fa5ba97aa8a46b1b5577.dat.local."
// [DEBUG trust_dns_proto::xfer::dns_handle] querying: 2e020eeb63d2767211d2fa5ba97aa8a46b1b5577.dat.local. TXT
// [DEBUG trust_dns_resolver::name_server::name_server] reconnecting: NameServerConfig { socket_addr: V4(224.0.0.251:5353), protocol: Mdns, tls_dns_name: None }
// [DEBUG trust_dns_proto::multicast::mdns_stream] binding sending stream to 0.0.0.0:29732
// [DEBUG trust_dns_proto::multicast::mdns_stream] preparing sender on: 0.0.0.0:29732
// [DEBUG trust_dns_proto::xfer] enqueueing message: [Query { name: Name { is_fqdn: true, labels: [2e020eeb63d2767211d2fa5ba97aa8a46b1b5577, dat, local] }, query_type: TXT, query_class: IN }]
///
///
fn client_trustdns(url: &str) {
  use trust_dns_client::client::{Client, SyncClient};
  use trust_dns_client::multicast::MdnsClientConnection;
  use trust_dns_client::op::DnsResponse;
  use trust_dns_client::rr::{DNSClass, Name, RecordType};

  let conn = MdnsClientConnection::new_ipv4(None, None);
  let client = SyncClient::new(conn);

  let name = Name::from_ascii(&url).expect("bad dns name");
  let response: DnsResponse = client
    .query(&name, DNSClass::IN, RecordType::TXT)
    .expect("could not query");

  dbg!(&response);
  dbg!(response.messages());
}

/// Also gets stuck sending request to mdns
fn async_trustdns(url: &str) {
  use tokio::runtime::Runtime;
  use trust_dns_client::rr::Name;
  use trust_dns_resolver::config::*;
  use trust_dns_resolver::AsyncResolver;

  let mut io_loop = Runtime::new().unwrap();
  let resolver = AsyncResolver::new(
    ResolverConfig::default(),
    ResolverOpts::default(),
    io_loop.handle().clone(),
  );

  let resolver = io_loop
    .block_on(resolver)
    .expect("failed to connect resolver");

  let name = Name::from_ascii(&url).expect("bad dns name");
  let lookup_future = resolver.txt_lookup(name);

  let response = io_loop.block_on(lookup_future).unwrap();

  dbg!(response);
}

/// Also gets stuck sending Request
fn sync_trustdns(url: &str) {
  use trust_dns_resolver::config::*;
  use trust_dns_resolver::Resolver;

  let resolver = Resolver::new(ResolverConfig::default(), ResolverOpts::default()).unwrap();
  let response = resolver.txt_lookup(dbg!(&url)).unwrap();
  dbg!(response);
}

fn main() -> Error {
  env_logger::builder().format_timestamp(None).init();
  let args: Vec<String> = std::env::args().skip(1).collect();
  let dat_name = args.first().expect("must have dat name");

  let dat_url = parse_dat_url::DatUrl::parse(dat_name)?;
  let public_key_bytes = hex::decode(dat_url.host().as_bytes()).expect("valid public key");
  let public_key = PublicKey::from_bytes(&public_key_bytes).expect("invalid key");
  let url = to_dns_discovery_key(&to_discovery_key(public_key)) + HOSTNAME;

  // mdns_lib(&url);
  // client_trustdns(&url);
  sync_trustdns(&url);
  // async_trustdns(&url);

  Ok(())
}
