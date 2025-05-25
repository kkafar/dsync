use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client_api_proto_path: PathBuf = "./proto/cli.proto".into();

    // Simulate behaviour of tonic_build::compile_protos

    let parent = client_api_proto_path
        .parent()
        .expect("expected a parent directory");

    tonic_build::configure()
        .out_dir("proto-generated/")
        .compile_protos(&[&client_api_proto_path], &[parent])?;
    Ok(())
}
