use std::{io::Read, path::Path};

use tokio::io::AsyncReadExt;

pub fn compute_sha1_hash_from_file(
    file_path: impl AsRef<Path>,
    buffer_read_capacity: Option<usize>,
) -> Result<String, anyhow::Error> {
    let path: &Path = file_path.as_ref();

    if !path.is_file() {
        anyhow::bail!("Provided file_path is not a regular file");
    }

    let mut file_handle = match std::fs::OpenOptions::new().read(true).open(path) {
        Ok(file_handle) => file_handle,
        Err(err) => {
            anyhow::bail!("Failed to open file: {path:?} with error: {err}");
        }
    };

    let buffer_capacity = buffer_read_capacity.unwrap_or(1024);
    let mut sha1_instance = sha1_smol::Sha1::new();
    let mut buffer: Vec<u8> = vec![0; buffer_capacity];

    // TODO: Will it work if the file is empty?
    while let Ok(bytes_read) = file_handle.read(&mut buffer) {
        if bytes_read == 0 {
            break;
        }
        sha1_instance.update(&buffer[0..bytes_read]);
    }

    anyhow::Ok(sha1_instance.digest().to_string())
}

pub async fn compute_sha1_hash_from_file_async(
    file_path: impl AsRef<Path>,
    buffer_read_capacity: Option<usize>,
) -> anyhow::Result<String> {
    let path: &Path = file_path.as_ref();

    if !path.is_file() {
        anyhow::bail!("Provided file_path is not a regular file");
    }

    let mut file_handle = match tokio::fs::OpenOptions::new().read(true).open(path).await {
        Ok(file_handle) => file_handle,
        Err(err) => {
            anyhow::bail!("Failed to open file: {path:?} with error: {err}");
        }
    };

    let buffer_capacity = buffer_read_capacity.unwrap_or(1024);
    let mut sha1_instance = sha1_smol::Sha1::new();
    let mut buffer: Vec<u8> = vec![0; buffer_capacity];

    // TODO: Will it work if the file is empty?
    while let Ok(bytes_read) = file_handle.read(&mut buffer).await {
        if bytes_read == 0 {
            break;
        }
        sha1_instance.update(&buffer[0..bytes_read]);
    }

    anyhow::Ok(sha1_instance.digest().to_string())
}

/// Basically tries to call `which ${binary_name}` & reports the command status.
/// Returns false if the check has failed for some other reason!
pub fn check_binary_exists(binary_name: &str) -> bool {
    let mut which_command = std::process::Command::new("which");
    which_command.arg(binary_name);

    let exit_status = match which_command.status() {
        Ok(status) => status,
        Err(err) => {
            println!(
                "Failed to determine whether the binary: {binary_name} exists with error: {err}"
            );
            return false;
        }
    };

    exit_status.success()
}
