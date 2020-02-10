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
    use syslog::{BasicLogger, Facility, Formatter3164};

    let formatter = Formatter3164 {
        facility: Facility::LOG_USER,
        hostname: None,
        process: "libcolmeia_dat1".into(),
        pid: 0,
    };

    let logger = syslog::unix(formatter).expect("could not connect to syslog");
    log::set_boxed_logger(Box::new(BasicLogger::new(logger)))
        .map(|()| log::set_max_level(LevelFilter::Debug));
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

    logging();
    let key = "dat://642b2da5e4267635259152eb0b1c04416030a891acd65d6c942b8227b8cbabed";
    let dat_key = colmeia_dat1_core::parse(&key).expect("invalid dat argument");
    let dat_key = match dat_key {
        colmeia_dat1_core::DatUrlResolution::HashUrl(result) => result,
        _ => panic!("invalid hash key"),
    };

    let mut dat = colmeia_dat1::Dat::in_memory(dat_key, "0.0.0.0:43898".parse().unwrap());
    dat.with_discovery(vec![dat.lan()]);
    log::warn!("Startig");
    async_std::task::spawn(async {
        let stream = std::panic::AssertUnwindSafe(dat.sync());
        let result = stream.catch_unwind().await;
        log::error!("Error {:?}", result);
    });
}
