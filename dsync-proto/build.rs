use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Simulate behaviour of tonic_build::compile_protos

    let proto_root_dir: PathBuf = "./proto".into();

    let user_agent_proto_path = proto_root_dir.join("user-agent.proto");
    let server_api_proto_path = proto_root_dir.join("server.proto");
    let shared_defs_proto_path = proto_root_dir.join("shared-defs.proto");

    let ft_messages = proto_root_dir.join("file-transfer/messages.proto");
    let ft_service = proto_root_dir.join("file-transfer/service.proto");

    tonic_build::configure()
        .out_dir("proto-generated/")
        .compile_protos(
            &[
                &user_agent_proto_path,
                &server_api_proto_path,
                &shared_defs_proto_path,
                &ft_messages,
                &ft_service,
            ],
            &[proto_root_dir],
        )?;
    Ok(())
}
