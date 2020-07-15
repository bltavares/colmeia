fn main() {
    let should_build = std::env::var("BUILD_PROTO").unwrap_or_default();

    if should_build == "build" {
        protobuf_codegen_pure::Codegen::new()
            .out_dir("src")
            .inputs(&["protos/schema.proto"])
            .includes(&["protos"])
            .run()
            .expect("protoc");
    };
}
