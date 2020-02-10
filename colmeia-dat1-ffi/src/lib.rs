#[cfg(target_os = "android")]
fn logging() {
    use android_logger::Config;
    use log::Level;
    android_logger::init_once(Config::default().with_min_level(Level::Debug));
}

#[no_mangle]
pub extern "C" fn sync() {
    logging();

    let key = "dat://642b2da5e4267635259152eb0b1c04416030a891acd65d6c942b8227b8cbabed";
    let dat_key = colmeia_dat1_core::parse(&key).expect("invalid dat argument");
    let dat_key = match dat_key {
        colmeia_dat1_core::DatUrlResolution::HashUrl(result) => result,
        _ => panic!("invalid hash key"),
    };

    let mut dat = colmeia_dat1::Dat::in_memory(dat_key, "0.0.0.0:43898".parse().unwrap());
    dat.with_discovery(vec![dat.lan()]);
    async_std::task::spawn(dat.sync());
}
