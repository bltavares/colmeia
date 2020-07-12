fn main() {
    let should_build = std::env::var("BUILD_PROTO").unwrap_or_default();

    if should_build == "build" {
        protobuf_codegen_pure::run(protobuf_codegen_pure::Args {
            out_dir: "src",
            input: &["protos/schema.proto"],
            includes: &["protos"],
            customize: protobuf_codegen_pure::Customize {
                ..Default::default()
            },
        })
        .expect("protoc");
    };
}
