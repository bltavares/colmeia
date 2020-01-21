use colmeia_mdns::Mdns;
use futures::stream::StreamExt;
use std::time::Duration;

fn name() -> String {
  let args: Vec<String> = std::env::args().skip(1).collect();
  args.first().expect("must have dat name as argument").into()
}

fn main() {
  env_logger::init();

  let name = name();

  async_std::task::block_on(async {
    let mut mdns = Mdns::new(&name).expect("could not parse DAT");
    mdns
      .with_announcer(3282)
      .with_location(Duration::from_secs(10));

    while let Some(peer) = mdns.next().await {
      println!("Peer found {:?}", peer);
    }
  })
}
