use async_std::net::TcpStream;
use async_std::stream::StreamExt;
use colmeia_dat_proto::*;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::PathBuf;

fn address() -> SocketAddr {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let input = args
        .first()
        .expect("must have dat server:port name as argument");
    input.parse().expect("invalid ip:port as input")
}

fn name() -> String {
    let args: Vec<String> = std::env::args().skip(2).collect();
    args.first().expect("must have dat name as argument").into()
}

fn folder() -> PathBuf {
    let args: Vec<String> = std::env::args().skip(3).collect();
    args.first().expect("must have folder as argument").into()
}

struct CloneDatClient<Storage>
where
    Storage: random_access_storage::RandomAccess<Error = failure::Error> + std::fmt::Debug,
{
    metadata: hypercore::Feed<Storage>,
    content: Option<hypercore::Feed<Storage>>,
    handshake: SimpleDatHandshake,
}

// TODO make it generic if possible
impl CloneDatClient<random_access_memory::RandomAccessMemory> {
    fn new(public_key: hypercore::PublicKey) -> Self {
        let metadata = hypercore::Feed::builder(
            public_key,
            hypercore::Storage::new_memory().expect("could not page metadata memory"),
        )
        .build()
        .expect("Could not start metadata");
        Self {
            metadata,
            content: None,
            handshake: SimpleDatHandshake::new(),
        }
    }

    fn initialize_content_feed(&mut self, public_key: hypercore::PublicKey) {
        if self.content.is_some() {
            panic!("double initialization of metadata")
        }

        self.content = Some(
            hypercore::Feed::builder(
                public_key,
                hypercore::Storage::new_memory().expect("could not page content memory"),
            )
            .build()
            .expect("Could not start content"),
        )
    }
}

// TODO make it generic
#[async_trait]
impl DatObserver for CloneDatClient<random_access_memory::RandomAccessMemory> {
    async fn on_feed(
        &mut self,
        client: &mut Client,
        channel: u64,
        message: &proto::Feed,
    ) -> Option<()> {
        self.handshake.on_feed(client, channel, message).await?;
        Some(())
    }

    async fn on_handshake(
        &mut self,
        client: &mut Client,
        channel: u64,
        message: &proto::Handshake,
    ) -> Option<()> {
        self.handshake
            .on_handshake(client, channel, message)
            .await?;
        Some(())
    }

    async fn on_finish(&mut self, _client: &mut Client) {
        println!("Metadata audit: {:?}", self.metadata.audit());
        println!("Conent audit: {:?}", self.content.unwrap().audit());
    }
}

// Content is another feed, with the pubkey derived from the metadata key
// https://github.com/mafintosh/hyperdrive/blob/e182fcbe6636a9c18fe4366b6ec3ce7c2f7f12a0/index.js#L955-L968
// const HYPERDRIVE: &[u8; 8] = b"hyperdri";
// fn derive_content_key() {
// What to do if blake2b and dalek don't have the KDF function?
// Only needed for creating metadata, as it needs the private key
// Readers only use what is provided on initialization.
// }

fn main() {
    env_logger::init();

    let address = address();
    let key = name();
    // let path = name();
    async_std::task::block_on(async {
        let tcp_stream = TcpStream::connect(address)
            .await
            .expect("could not open address");
        let client_initialization = new_client(&key, tcp_stream).await;
        let client = handshake(client_initialization)
            .await
            .expect("could not handshake");

        // let observer = SimpleDatClient::new();
        // let mut service = DatService::new(client, observer);

        // while let Some(message) = service.next().await {
        //     if let DatMessage::Feed(message) = message.parse().unwrap() {
        //         eprintln!(
        //             "Received message {:?}",
        //             hex::encode(message.get_discoveryKey())
        //         );
        //     }
        // }
    });
}
