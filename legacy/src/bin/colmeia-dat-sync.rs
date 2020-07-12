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
    env_logger::init();

    let key = name();
    let dat_key = colmeia_dat1_core::parse(&key).expect("invalid dat argument");
    let dat_key = match dat_key {
        colmeia_dat1_core::DatUrlResolution::HashUrl(result) => result,
        _ => panic!("invalid hash key"),
    };

    let mut dat = colmeia_dat1::Dat::in_memory(dat_key, "0.0.0.0:3899".parse().unwrap())
        .expect("could not start dat");
    dat.with_discovery(dat.lan());

    async_std::task::block_on(dat.sync());
}
