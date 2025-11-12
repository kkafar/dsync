use std::{
    error::Error,
    fs,
    path::{Path, PathBuf},
};

fn extract_proto_files_from_dir_recursively(
    dir_path: impl AsRef<Path>,
) -> Result<Vec<String>, Box<dyn Error>> {
    let result: Vec<String> = fs::read_dir(dir_path)?
        .filter_map(Result::ok)
        .flat_map(|entry| {
            println!("Handling path: {}", entry.file_name().to_str().unwrap());
            if entry
                .metadata()
                .expect("Failed to read file metadata")
                .is_dir()
            {
                return extract_proto_files_from_dir_recursively(entry.path()).unwrap();
            } else if entry.file_name().to_str().unwrap().ends_with(".proto")
                && entry.file_name().to_str().unwrap().contains("service")
            {
                return vec![entry.path().to_str().unwrap().to_owned()];
            } else {
                return Vec::new();
            }
        })
        .collect();

    return Ok(result);
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Simulate behaviour of tonic_build::compile_protos

    let proto_root_dir: PathBuf = "./proto".into();

    if !proto_root_dir.is_dir() {
        return Err("\"./proto/\" is not a directory!".into());
    }

    // let user_agent_proto_path = proto_root_dir.join("user-agent.proto");
    // let server_api_proto_path = proto_root_dir.join("server.proto");
    // let shared_defs_proto_path = proto_root_dir.join("shared-defs.proto");
    //
    // let ft_messages = proto_root_dir.join("file-transfer/messages.proto");
    // let ft_service = proto_root_dir.join("file-transfer/service.proto");

    let proto_files = extract_proto_files_from_dir_recursively(&proto_root_dir)?;

    for path_entry in proto_files.iter() {
        println!("Compiled files: {}", path_entry);
    }

    tonic_build::configure()
        .out_dir("proto-generated/")
        .compile_protos(&proto_files, &[proto_root_dir])?;

    // tonic_build::configure()
    //     .out_dir("proto-generated/")
    //     .compile_protos(
    //         &[
    //             &user_agent_proto_path,
    //             &server_api_proto_path,
    //             &shared_defs_proto_path,
    //             &ft_messages,
    //             &ft_service,
    //         ],
    //         &[proto_root_dir],
    //     )?;
    Ok(())
}
