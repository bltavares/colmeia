use anyhow::Context;
use async_std::{sync::RwLock, task};
use futures::{Future, SinkExt, Stream, StreamExt};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;
use std::{collections::HashMap, io};
use trust_dns_proto::op::{Message, MessageType, Query};
use trust_dns_proto::rr::{Name, RData, RecordType};
use trust_dns_proto::serialize::binary::{BinEncodable, BinEncoder};

pub fn packet(hyperswarm_domain: Name) -> anyhow::Result<Vec<u8>> {
    let query = Query::query(hyperswarm_domain, RecordType::SRV);
    let mut message = Message::new();
    message
        .set_id(0)
        .set_message_type(MessageType::Query)
        .set_authoritative(false)
        .add_query(query);

    let mut buffer = Vec::with_capacity(75);
    let mut encoder = BinEncoder::new(&mut buffer);
    message
        .emit(&mut encoder)
        .context("malformed mdns packet")?;
    Ok(buffer)
}

async fn broadcast(
    hyperswarm_domain: Name,
    socket: Arc<multicast_socket::MulticastSocket>,
) -> anyhow::Result<()> {
    let mdns_packet_bytes = packet(hyperswarm_domain)?;
    task::spawn(async move { socket.broadcast(&mdns_packet_bytes) })
        .await
        .context("could not send packet to multicast address")?;

    Ok(())
}

async fn wait_response(
    socket: Arc<multicast_socket::MulticastSocket>,
) -> io::Result<multicast_socket::Message> {
    task::spawn(async move { socket.receive() }).await
}

pub struct Locator {
    topics: Arc<RwLock<HashMap<Vec<u8>, Name>>>,
    _listen_task: task::JoinHandle<()>,
    _broadcast_task: task::JoinHandle<()>,
    stream: Box<dyn Stream<Item = (Vec<u8>, SocketAddr)> + Unpin + Send + Sync>,
}

impl Locator {
    pub fn listen(
        socket: multicast_socket::MulticastSocket,
        duration: Duration,
        self_id: &[u8],
    ) -> Self {
        let topics: Arc<RwLock<HashMap<Vec<u8>, Name>>> = Default::default();
        let socket = Arc::new(socket);

        let broadcast_task = {
            let topics = topics.clone();
            let socket = socket.clone();

            task::spawn(async move {
                loop {
                    for (_, topic) in topics.read().await.iter() {
                        let broadcast_result = broadcast(topic.clone(), socket.clone()).await;
                        if let Err(problem) = broadcast_result {
                            log::warn!(
                                "failed to broadcast a packet. trying again later. {:?}",
                                problem
                            );
                        }
                    }
                    task::sleep(duration).await;
                }
            })
        };

        let (mut sender, receiver) = futures::channel::mpsc::unbounded();
        let listen_task = {
            let topics = topics.clone();
            let self_id = [self_id.to_vec().into_boxed_slice()];

            task::spawn(async move {
                loop {
                    if let Ok(message) = wait_response(socket.clone()).await {
                        for (discovery_key, hyperswarm_domain) in topics.read().await.iter() {
                            let found = select_ip_from_hyperswarm_mdns_response(
                                &message.data,
                                message.origin_address.ip(),
                                &hyperswarm_domain,
                                &self_id,
                            );

                            if let Some(peer) = found {
                                let result = sender.send((discovery_key.clone(), peer)).await;
                                log::debug!("Announce received {:?}: {:?}", peer, result);
                            }
                        }
                    }
                }
            })
        };

        Self {
            topics,
            _broadcast_task: broadcast_task,
            _listen_task: listen_task,
            stream: Box::new(receiver),
        }
    }

    pub fn add_topic(&self, topic: Vec<u8>) -> impl Future<Output = anyhow::Result<()>> {
        let topics = self.topics.clone();

        async move {
            let value = crate::hash_as_domain_name(&topic)?;
            topics.write().await.insert(topic, value);
            Ok(())
        }
    }

    pub fn remove_topic(&self, topic: Vec<u8>) -> impl Future<Output = anyhow::Result<()>> {
        let topics = self.topics.clone();
        async move {
            topics.write().await.remove(&topic);
            Ok(())
        }
    }
}

fn select_ip_from_hyperswarm_mdns_response(
    packet: &[u8],
    origin_ip: &Ipv4Addr,
    hyperswarm_domain: &Name,
    self_id: crate::SelfId,
) -> Option<SocketAddr> {
    let dns_message = Message::from_vec(packet).ok()?;
    if dns_message.answer_count() != 3 {
        return None;
    }

    let answers = dns_message.answers();
    let _id_different_from_self = answers.iter().find(|record| {
        if let RData::TXT(txt_data) = record.rdata() {
            return txt_data.txt_data() != self_id;
        }
        false
    })?;

    let srv_matches = answers
        .iter()
        .find(|record| record.name() == hyperswarm_domain)?;
    if let RData::SRV(srv_data) = srv_matches.rdata() {
        let port = srv_data.port();
        if srv_data.target() == &*crate::UNSPECIFIED_NAME {
            return Some(SocketAddr::new(IpAddr::V4(*origin_ip), port));
        }

        let target_ipv4 = srv_data.target().to_utf8().parse::<Ipv4Addr>().ok()?;
        return Some(SocketAddr::new(IpAddr::V4(target_ipv4), port));
    }
    None
}

impl futures::Stream for Locator {
    type Item = (Vec<u8>, SocketAddr);
    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut task::Context<'_>,
    ) -> task::Poll<Option<Self::Item>> {
        self.stream.poll_next_unpin(cx)
    }
}
