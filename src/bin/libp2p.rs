use async_std::prelude::*;
use libp2p::mdns::MdnsEvent::Discovered;
use libp2p::multiaddr::multiaddr;

/// Possible to find other nodes if I start the cli twice.
/// Questions:
/// But it is hard to figure out how to integrate with hypwerswarm (new NetworkBehaviour?)
/// Would be nice to add a dat on libp2p, but it would not integrate with hyperswarm.
/// libp2p requires both sides to be libp2p? (eg: peerid and upgrade/connection)
/// dat keypair as libp2p keypair?
fn main() {
    async_std::task::block_on(async {
        let keypair = libp2p::identity::Keypair::generate_ed25519();
        let peer_id = keypair.public().into_peer_id();
        let _transport = libp2p::build_development_transport(keypair).expect("create the infra");
        let _behaviour = libp2p::mdns::Mdns::new()
            .await
            .expect("could not start anything");
        let mut swarm = libp2p::Swarm::new(_transport, _behaviour, peer_id);

        libp2p::Swarm::listen_on(&mut swarm, multiaddr!(Ip4([0, 0, 0, 0]), Tcp(10500u16)));

        while let Some(x) = swarm.next().await {
            if let Ok(Discovered(y)) = x {
                dbg!(y.len());
                for (peer_id, address) in y {
                    dbg!(peer_id);
                    dbg!(address);
                }
            }
        }
    });
}
