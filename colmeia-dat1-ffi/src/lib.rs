use futures::future::FutureExt;
use std::panic;

#[cfg(target_os = "android")]
fn logging() {
    use android_logger::Config;
    use log::Level;
    android_logger::init_once(Config::default().with_min_level(Level::Debug));
}

#[cfg(target_os = "ios")]
fn logging() {
    use log::LevelFilter;
    use syslog::Facility;

    syslog::init_unix(Facility::LOG_USER, LevelFilter::max()).expect("could not connect to syslog");
}

#[cfg(all(not(target_os = "ios"), not(target_os = "android")))]
fn logging() {}

#[no_mangle]
pub extern "C" fn colmeia_dat1_sync() {
    logging();

    panic::set_hook(Box::new(|e| {
        log::error!("Error: {:?}", e);
        println!("Custom panic hook");
    }));

    let key = "dat://6268b99fbacacea49c6bc3d4776b606db2aeadb3fa831342ba9f70d55c98929f";
    let dat_key = colmeia_dat1_core::parse(&key).expect("invalid dat argument");
    let dat_key = match dat_key {
        colmeia_dat1_core::DatUrlResolution::HashUrl(result) => result,
        _ => panic!("invalid hash key"),
    };

    let mut dat = colmeia_dat1::Dat::in_memory(dat_key, "0.0.0.0:43898".parse().unwrap());
    dat.with_discovery(vec![dat.lan()]);
    log::warn!("Starting");
    async_std::task::spawn(async {
        let stream = std::panic::AssertUnwindSafe(dat.sync());
        let result = stream.catch_unwind().await;
        log::error!("Error {:?}", result);
    });
}
