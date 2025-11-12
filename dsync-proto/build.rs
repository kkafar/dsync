use std::{
    error::Error,
    fs,
    path::{Path, PathBuf},
};

fn extract_proto_files_from_dir_recursively(
    dir_path: impl AsRef<Path>,
) -> Result<Vec<String>, Box<dyn Error>> {
    let mut proto_files: Vec<String> = Vec::new();

    for entry in fs::read_dir(dir_path)?.filter_map(Result::ok) {
        if entry
            .metadata()
            .expect("Failed to read file metadata")
            .is_dir()
        {
            let dir_result = extract_proto_files_from_dir_recursively(entry.path())?;
            proto_files.extend_from_slice(&dir_result);
        } else if entry.file_name().to_str().unwrap().ends_with(".proto") {
            proto_files.push(entry.path().to_str().unwrap().to_owned());
        }
    }

    return Ok(proto_files);
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Simulate behaviour of tonic_build::compile_protos

    let proto_root_dir: PathBuf = "./proto".into();

    if !proto_root_dir.is_dir() {
        return Err("\"./proto/\" is not a directory!".into());
    }

    let proto_files = extract_proto_files_from_dir_recursively(&proto_root_dir)?;
    let service_files: Vec<String> = proto_files
        .into_iter()
        // Take only services - naming convention required
        .filter(|proto_file| proto_file.ends_with("service.proto"))
        .collect();

    tonic_build::configure()
        .out_dir("proto-generated/")
        .compile_protos(&service_files, &[proto_root_dir])?;

    Ok(())
}
