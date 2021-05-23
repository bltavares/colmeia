use async_std::{prelude::StreamExt, task};
use colmeia_hyperstack::{utils::PublicKeyExt, Hyperstack};

fn name() -> String {
    let args: Vec<String> = std::env::args().skip(1).collect();
    args.first().expect("must have dat name as argument").into()
}

// TODO: send to a folder
// use std::path::PathBuf;
// fn folder() -> PathBuf {
//     let args: Vec<String> = std::env::args().skip(3).collect();
//     args.first().expect("must have folder as argument").into()
// }

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let key = name();
    let hash = key.parse_from_hash().expect("invalid dat argument");

    // let path = name();
    task::block_on(async {
        let mut hyperstack = Hyperstack::in_memory(hash, "0.0.0.0:3899".parse().unwrap())
            .await
            .expect("Could not start hyperdrive on the stack");
        let mdns = hyperstack.lan().await.expect("could not configure mdns");
        let dht = hyperstack.dht().await.expect("could not start dht");
        hyperstack.with_discovery(mdns.merge(dht));
        hyperstack.replicate().await;
    });
}
