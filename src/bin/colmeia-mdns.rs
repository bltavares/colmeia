use async_std::prelude::FutureExt;
use colmeia_mdns::Announcer;
use colmeia_mdns::Locator;
use futures::stream::StreamExt;
use std::time::Duration;

fn name() -> String {
  let args: Vec<String> = std::env::args().skip(1).collect();
  args.first().expect("must have dat name as argument").into()
}

fn main() {
  env_logger::init();

  let name = name();
  let url_name = colmeia_mdns::crypto::dat_url_mdns_discovery_name(&name)
    .expect("cold not convert to dat local url");

  async_std::task::block_on(async {
    let discovery = async_std::task::spawn(async {
      let mut locator = Locator::new(
        colmeia_mdns::create_shared_socket().expect("socket creation failed"),
        url_name,
        Duration::from_secs(10),
      );
      while let Some(message) = locator.next().await {
        println!("dat found on: {:?}", message);
      }
    });

    let announcement = async_std::task::spawn(async {
      let mut announcer =
        Announcer::new(colmeia_mdns::create_shared_socket().expect("socket creation failed"));
      while let Some(message) = announcer.next().await {
        println!("dat found on: {:?}", message);
      }
    });

    discovery.join(announcement).await;
  });
}
