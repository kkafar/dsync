use std::{
    io::Read,
    path::{Path, PathBuf},
};

pub(super) fn compute_sha1_hash_from_file(
    file_path: impl AsRef<Path>,
    buffer_read_capacity: Option<usize>,
) -> anyhow::Result<String> {
    let path: &Path = file_path.as_ref();

    if !path.is_file() {
        return Err(anyhow::anyhow!("Provided file_path is not a regular file"));
    }

    let mut file_handle = match std::fs::OpenOptions::new().read(true).open(path) {
        Ok(file_handle) => file_handle,
        Err(err) => {
            return Err(anyhow::anyhow!(format!(
                "Failed to open file: {path:?} with error: {err}"
            )));
        }
    };

    let mut sha1_instance = sha1_smol::Sha1::new();
    let mut buffer: Vec<u8> = Vec::with_capacity(buffer_read_capacity.unwrap_or(1024));

    // TODO: Will it work if the file is empty?
    while let Ok(bytes_read) = file_handle.read(&mut buffer) {
        if bytes_read == 0 {
            break;
        }
        sha1_instance.update(&buffer[0..bytes_read]);
    }

    anyhow::Ok(sha1_instance.digest().to_string())
}
