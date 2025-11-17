use std::path::PathBuf;

use dsync_proto::services::user_agent::{FileSourceSpec, HostSpec, PathSpec, host_spec, path_spec};

#[derive(Debug, thiserror::Error)]
pub enum FileSourceSpecParseError {
    #[error("Invalid origin host specification")]
    InvalidOriginHostSpec,

    #[error("Invalid destination host specification")]
    InvalidDestHostSpec,

    #[error("Invalid file path specification")]
    InvalidSpec,
}

pub struct PathSpecWrapper(pub path_spec::Inner);

impl PathSpecWrapper {
    pub fn try_into_abs_path(self) -> anyhow::Result<String> {
        Ok(match self.0 {
            path_spec::Inner::AbsolutePath(abs_path) => abs_path,
            path_spec::Inner::RelativePath(rel_path) => {
                std::path::absolute(PathBuf::from(rel_path))?
                    .to_str()
                    .ok_or(anyhow::anyhow!(
                        "Failed to convert absolute path to string for relative path"
                    ))?
                    .to_owned()
            }
        })
    }

    pub fn into_direct_string(self) -> String {
        match self.0 {
            path_spec::Inner::AbsolutePath(abs_path) => abs_path,
            path_spec::Inner::RelativePath(rel_path) => rel_path,
        }
    }
}

impl From<path_spec::Inner> for PathSpecWrapper {
    fn from(value: path_spec::Inner) -> Self {
        Self(value)
    }
}

pub struct HostSpecWrapper(pub host_spec::Inner);

impl HostSpecWrapper {
    pub fn into_host_spec_string(self) -> String {
        match self.0 {
            host_spec::Inner::LocalHost(_) => "localhost".to_owned(),
            host_spec::Inner::Name(name) => name,
            host_spec::Inner::LocalId(id) => id.to_string(),
        }
    }
}

impl From<host_spec::Inner> for HostSpecWrapper {
    fn from(value: host_spec::Inner) -> Self {
        Self(value)
    }
}

pub struct FileSourceSpecWrapper {
    pub host: HostSpecWrapper,
    pub path: PathSpecWrapper,
}

impl From<FileSourceSpec> for FileSourceSpecWrapper {
    fn from(value: FileSourceSpec) -> Self {
        Self {
            host: HostSpecWrapper(value.host_spec.unwrap().inner.unwrap()),
            path: PathSpecWrapper(value.path_spec.unwrap().inner.unwrap()),
        }
    }
}

impl Into<FileSourceSpec> for FileSourceSpecWrapper {
    fn into(self) -> FileSourceSpec {
        FileSourceSpec {
            host_spec: Some(HostSpec {
                inner: Some(self.host.0),
            }),
            path_spec: Some(PathSpec {
                inner: Some(self.path.0),
            }),
        }
    }
}

// [HOST@]FILE_PATH
pub fn parse_file_source_spec(
    spec: impl AsRef<str>,
) -> Result<FileSourceSpec, FileSourceSpecParseError> {
    let spec_str = spec.as_ref();

    if !spec_str.contains("@") {
        return Ok(FileSourceSpec {
            host_spec: Some(parse_file_source_host_spec(spec_str)?),
            path_spec: Some(parse_file_source_path_spec(spec_str)?),
        });
    }

    let mut split_str = spec_str.splitn(2, "@");
    let host_part = split_str
        .next()
        .ok_or(FileSourceSpecParseError::InvalidSpec)?;
    let file_part = split_str
        .next()
        .ok_or(FileSourceSpecParseError::InvalidSpec)?;

    Ok(FileSourceSpec {
        host_spec: Some(parse_file_source_host_spec(&host_part)?),
        path_spec: Some(parse_file_source_path_spec(&file_part)?),
    })
}

fn parse_file_source_path_spec(spec: &str) -> Result<PathSpec, FileSourceSpecParseError> {
    let mut path_buf = PathBuf::from(spec);
    if path_buf.is_absolute() {
        Ok(PathSpec::AbsolutePath(spec.to_owned()))
    } else {
        Ok(PathSpec::RelativePath(spec.to_owned()))
    }
}

fn parse_file_source_host_spec(spec: &str) -> Result<HostSpec, FileSourceSpecParseError> {
    if spec == "localhost" {
        return Ok(HostSpec::LocalHost(0));
    }

    let maybe_id = spec.parse::<i32>();
    if let Ok(host_local_id) = maybe_id {
        return Ok(HostSpec::LocalId(host_local_id));
    }

    return Ok(HostSpec::Name(spec.to_owned()));
}
