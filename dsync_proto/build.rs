use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Simulate behaviour of tonic_build::compile_protos

    let proto_root_dir: PathBuf = "./proto".into();

    let client_api_proto_path = proto_root_dir.join("cli.proto");
    let server_api_proto_path = proto_root_dir.join("server.proto");

    tonic_build::configure()
        .out_dir("proto-generated/")
        .compile_protos(
            &[&client_api_proto_path, &server_api_proto_path],
            &[proto_root_dir],
        )?;
    Ok(())
}
