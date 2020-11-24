use anyhow::Context;
use async_std::{sync::RwLock, task};
use futures::{stream::StreamExt as FStreamExt, SinkExt, Stream};
use trust_dns_proto::op::{Message, MessageType};
use trust_dns_proto::rr::{
    rdata::{SRV, TXT},
    Name, RData, Record, RecordType,
};
use trust_dns_proto::serialize::binary::{BinEncodable, BinEncoder};

use std::sync::Arc;
use std::{collections::HashMap, io};
use std::{net::Ipv4Addr, pin::Pin};

pub fn packet(
    hyperswarm_domain: Name,
    port: u16,
    self_identifier: String,
) -> anyhow::Result<Vec<u8>> {
    let mut srv_query = Record::with(hyperswarm_domain.clone(), RecordType::SRV, 0);
    srv_query.set_rdata(RData::SRV(SRV::new(
        0,
        0,
        port,
        crate::UNSPECIFIED_NAME.clone(),
    )));

    let mut txt_query = Record::with(hyperswarm_domain, RecordType::TXT, 0);
    txt_query.set_rdata(RData::TXT(TXT::new(vec![self_identifier])));

    let mut a_query = Record::with(crate::HYPERSWARM_REFERER.clone(), RecordType::A, 0);
    a_query.set_rdata(RData::A(Ipv4Addr::UNSPECIFIED));

    let mut message = Message::new();
    message
        .set_id(0)
        .set_message_type(MessageType::Response)
        .set_authoritative(false)
        .add_answer(srv_query)
        .add_answer(txt_query)
        .add_answer(a_query);

    let mut buffer = Vec::with_capacity(75);
    let mut encoder = BinEncoder::new(&mut buffer);
    message.emit(&mut encoder).context("could not encode")?;
    Ok(buffer)
}

async fn respond(
    hyperswarm_domain: Name,
    interface: multicast_socket::Interface,
    socket: Arc<multicast_socket::MulticastSocket>,
    port: u16,
    self_identifier: String,
) -> anyhow::Result<()> {
    let mdns_packet_bytes = packet(hyperswarm_domain, port, self_identifier)?;

    task::spawn(async move { socket.send(&mdns_packet_bytes, &interface) })
        .await
        .context("Could not send bytes")?;

    Ok(())
}

async fn wait_broadcast(
    socket: Arc<multicast_socket::MulticastSocket>,
) -> io::Result<multicast_socket::Message> {
    task::spawn(async move { socket.receive() }).await
}

fn is_same_hash_questions(packet: &[u8], hyperswarm_domain: &Name) -> Option<Message> {
    let dns_message = Message::from_vec(packet).ok()?;
    if dns_message.query_count() != 1 {
        return None;
    }

    let query = dns_message.queries().first()?;
    if query.query_type() != RecordType::SRV {
        return None;
    }

    if query.name() != hyperswarm_domain {
        return None;
    }

    Some(dns_message)
}

pub struct Announcer {
    topics: Arc<RwLock<HashMap<Vec<u8>, Name>>>,
    _listener_job: task::JoinHandle<()>,
    stream: Box<dyn Stream<Item = (Vec<u8>, Ipv4Addr)> + Unpin + Send + Sync>,
}

impl Announcer {
    pub fn listen(
        socket: multicast_socket::MulticastSocket,
        port: u16,
        self_identifier: String,
    ) -> Self {
        let topics: Arc<RwLock<HashMap<Vec<u8>, Name>>> = Default::default();
        let socket = Arc::from(socket);

        let (mut sender, receiver) = futures::channel::mpsc::unbounded();
        let listener_job = {
            let topics = topics.clone();

            task::spawn(async move {
                loop {
                    if let Ok(message) = wait_broadcast(socket.clone()).await {
                        if let Some((discovery_key, name)) =
                            topics.read().await.iter().find(|(_, name)| {
                                is_same_hash_questions(&message.data, name).is_some()
                            })
                        {
                            let result = sender
                                .send((discovery_key.clone(), *message.origin_address.ip()))
                                .await;
                            log::debug!("Announce received {:?}", result);
                            let reply = respond(
                                (*name).clone(),
                                message.interface,
                                socket.clone(),
                                port,
                                self_identifier.clone(),
                            )
                            .await;

                            if let Err(e) = reply {
                                log::warn!("Could not send response back {}", e);
                            }
                        }
                    }
                }
            })
        };

        Self {
            topics,
            _listener_job: listener_job,
            stream: Box::new(receiver),
        }
    }

    pub async fn add_topic(&self, topic: &[u8]) -> anyhow::Result<()> {
        let value = crate::hash_as_domain_name(topic)?;
        self.topics.write().await.insert(topic.to_vec(), value);
        Ok(())
    }

    pub async fn remove_topic(&self, topic: &[u8]) -> anyhow::Result<()> {
        self.topics.write().await.remove(topic);
        Ok(())
    }
}

impl futures::Stream for Announcer {
    type Item = (Vec<u8>, Ipv4Addr);
    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut task::Context<'_>,
    ) -> task::Poll<Option<Self::Item>> {
        self.stream.poll_next_unpin(cx)
    }
}
