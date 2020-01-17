fn name() -> String {
  let args: Vec<String> = std::env::args().skip(1).collect();
  args.first().expect("must have dat name").into()
}

use colmeia_mdns::Locator;
use futures::stream::StreamExt;
use std::time::Duration;

fn main() {
  let socket = colmeia_mdns::create_shared_socket().expect("socket creation failed");
  let name = name();
  let url_name = colmeia_mdns::crypto::dat_url_mdns_discovery_name(&name)
    .expect("cold not convert to dat local url");

  async_std::task::block_on(async {
    let mut locator = Locator::new(socket, url_name, Duration::from_secs(10));
    while let Some(message) = locator.next().await {
      dbg!(message);
      dbg!("message received");
    }
  });
}
