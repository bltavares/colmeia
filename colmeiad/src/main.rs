use async_std::{sync::RwLock, task};
use colmeia_hypercore::*;
use futures::future::OptionFuture;
use std::sync::Arc;
use tide::{Request, StatusCode};

fn name() -> String {
    let args: Vec<String> = std::env::args().skip(1).collect();
    args.first()
        .expect("must have hash name as argument")
        .into()
}

#[derive(serde::Serialize, Debug)]
struct FeedInfo {
    len: u64,
    blocks: u64,
}

#[derive(serde::Serialize, Debug)]
struct Info {
    metadata: FeedInfo,
    content: Option<FeedInfo>,
}

type State<Storage> = Arc<RwLock<Hyperdrive<Storage>>>;

async fn get_info<Storage>(req: Request<State<Storage>>) -> tide::Result<tide::Response>
where
    Storage: random_access_storage::RandomAccess<Error = Box<dyn std::error::Error + Send + Sync>>
        + std::fmt::Debug
        + Send
        + Sync,
{
    let driver = req.state().read().await;

    let content: OptionFuture<_> = driver.content.as_ref().map(feed_info).into();
    let content = content.await.transpose()?;

    let metadata = feed_info(&driver.metadata).await?;

    let info = Info { metadata, content };
    Ok(tide::Response::builder(200)
        .body(tide::convert::json!(info))
        .build())
}

async fn feed_info<Storage>(feed: &Arc<RwLock<hypercore::Feed<Storage>>>) -> tide::Result<FeedInfo>
where
    Storage: random_access_storage::RandomAccess<Error = Box<dyn std::error::Error + Send + Sync>>
        + std::fmt::Debug
        + Send
        + Sync,
{
    let len = feed.read().await.len();
    let blocks = feed
        .write()
        .await
        .audit()
        .await
        .map_err(|e| tide::Error::from_str(StatusCode::InternalServerError, e))?
        .valid_blocks;
    Ok(FeedInfo { len, blocks })
}

#[async_std::main]
async fn main() -> Result<(), std::io::Error> {
    env_logger::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let key = name();
    let hash = key.parse_from_hash().expect("invalid hash argument");

    let mut hyperstack = Hyperstack::in_memory(hash, "0.0.0.0:3899".parse().unwrap())
        .await
        .expect("Could not start hyperdrive on the stack");
    let mdns = hyperstack
        .lan()
        .await
        .expect("could not add key to mdns discovery");
    hyperstack.with_discovery(mdns);

    let job = task::spawn(hyperstack.replicate());
    let hyperdrive = hyperstack.hyperdrive();

    let mut app = tide::with_state(hyperdrive);
    app.middleware(tide::log::LogMiddleware::new());
    app.at("/").get(get_info);
    app.listen("127.0.0.1:8080").await?;
    job.await;
    Ok(())
}
