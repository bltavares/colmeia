use async_std::{prelude::FutureExt, prelude::StreamExt, sync::RwLock, task};
use colmeia_hypercore::PublicKeyExt;
use colmeia_hyperswarm_mdns::MdnsDiscovery;
use std::{env, sync::Arc, time::Duration};

// 7e5998407b3d9dbb94db21ff50ad6f1b1d2c79e476fbaf9856c342eb4382e7f5
// pra
// b62ef6792a0e2f2c5be328593532415d80c0f52c // 32 bytes
// Derivacao igual ao dat1
fn main() {
    log::debug!("Starting up");
    let hyper_hash = env::args()
        .nth(1)
        .expect("provide a hyper hash as argument");

    let port = env::args()
        .nth(2)
        .expect("requires a port as second argument")
        .parse::<u16>()
        .expect("Could not parse port as number");

    let duration = env::args()
        .nth(3)
        .unwrap_or_else(|| "10".into())
        .parse::<u64>()
        .expect("Could not parse duration into a number");

    env_logger::init();

    let key = hyper_hash.parse_from_hash().expect("could not parse hash");

    task::block_on(async move {
        let mdns = Arc::new(RwLock::new(MdnsDiscovery::new()));
        let topic = hypercore_protocol::discovery_key(key.as_bytes());
        mdns.write()
            .await
            .with_announcer(port)
            .with_locator(Duration::from_secs(duration));

        // Spawn a task to print a topic
        {
            let mdns = mdns.clone();
            task::spawn(async move {
                loop {
                    log::debug!("trying to find a peer");
                    let discovery_attempt = {
                        let mut write_lock = mdns.write().await;
                        write_lock.next().timeout(Duration::from_secs(1)).await
                    };

                    match discovery_attempt {
                        Ok(Some((discovery_key, peer))) => {
                            println!(
                                "Found peer {:?} serving {}",
                                peer,
                                hex::encode(discovery_key)
                            );
                        }
                        Err(_) => {
                            task::yield_now().await;
                        }
                        Ok(None) => {
                            break;
                        }
                    }
                }
            });
        }

        // Spawn a task to add a topic
        {
            let mdns = mdns.clone();
            let topic = topic.clone();
            task::spawn(async move {
                log::debug!("registering topic");
                mdns.write()
                    .await
                    .add_topic(&topic)
                    .await
                    .expect("could not write topic");
                log::debug!("topic registered");
            });
        }

        // Spawn a task to remove a topic after 30 secs, then finish the program
        task::spawn(async move {
            task::sleep(Duration::from_secs(30)).await;
            log::debug!("sopping cli");
            mdns.write()
                .await
                .remove_topic(&topic)
                .await
                .expect("failed to remove topic");
            log::debug!("stoped cli");
        })
        .await;
    });
}
