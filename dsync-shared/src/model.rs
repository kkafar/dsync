#![allow(unused)]

use std::path::PathBuf;

use dsync_proto::services::user_agent::{
    FileSource,
    file_source::{self, HostSpec, PathSpec},
};

#[derive(Debug, thiserror::Error)]
pub enum FileSourceParseError {
    // #[error("Invalid origin host specification")]
    // InvalidOriginHostSpec,
    //
    // #[error("Invalid destination host specification")]
    // InvalidDestHostSpec,
    #[error("Invalid file path specification")]
    InvalidSpec,
}

pub struct PathSpecWrapper(pub PathSpec);

impl PathSpecWrapper {
    pub fn try_into_abs_path(self) -> anyhow::Result<String> {
        Ok(match self.0 {
            PathSpec::AbsolutePath(abs_path) => abs_path,
            PathSpec::RelativePath(rel_path) => std::path::absolute(PathBuf::from(rel_path))?
                .to_str()
                .ok_or(anyhow::anyhow!(
                    "Failed to convert absolute path to string for relative path"
                ))?
                .to_owned(),
        })
    }

    pub fn into_direct_string(self) -> String {
        match self.0 {
            PathSpec::AbsolutePath(abs_path) => abs_path,
            PathSpec::RelativePath(rel_path) => rel_path,
        }
    }
}

impl From<PathSpec> for PathSpecWrapper {
    fn from(value: PathSpec) -> Self {
        Self(value)
    }
}

pub struct HostSpecWrapper(pub HostSpec);

impl HostSpecWrapper {
    pub fn into_host_spec_string(self) -> String {
        match self.0 {
            HostSpec::LocalHost(_) => "localhost".to_owned(),
            HostSpec::Name(name) => name,
            HostSpec::LocalId(id) => id.to_string(),
        }
    }
}

impl From<HostSpec> for HostSpecWrapper {
    fn from(value: HostSpec) -> Self {
        Self(value)
    }
}

pub struct FileSourceWrapper {
    pub host_spec: HostSpecWrapper,
    pub path_spec: PathSpecWrapper,
}

impl From<FileSource> for FileSourceWrapper {
    fn from(value: FileSource) -> Self {
        Self {
            host_spec: HostSpecWrapper(value.host_spec.expect("host_spec field is required")),
            path_spec: PathSpecWrapper(value.path_spec.expect("path_spec field is required")),
        }
    }
}

impl Into<FileSource> for FileSourceWrapper {
    fn into(self) -> FileSource {
        FileSource {
            host_spec: Some(self.host_spec.0),
            path_spec: Some(self.path_spec.0),
        }
    }
}

// [HOST@]FILE_PATH
pub fn parse_file_source_spec(spec: impl AsRef<str>) -> Result<FileSource, FileSourceParseError> {
    let spec_str = spec.as_ref();

    if !spec_str.contains("@") {
        return Ok(FileSource {
            host_spec: Some(parse_file_source_host_spec("localhost")?),
            path_spec: Some(parse_file_source_path_spec(spec_str)?),
        });
    }

    let mut split_str = spec_str.splitn(2, "@");
    let host_part = split_str.next().ok_or(FileSourceParseError::InvalidSpec)?;
    let file_part = split_str.next().ok_or(FileSourceParseError::InvalidSpec)?;

    Ok(FileSource {
        host_spec: Some(parse_file_source_host_spec(&host_part)?),
        path_spec: Some(parse_file_source_path_spec(&file_part)?),
    })
}

pub fn parse_file_source_path_spec(spec: &str) -> Result<PathSpec, FileSourceParseError> {
    let path_buf = PathBuf::from(spec);
    if path_buf.is_absolute() {
        Ok(PathSpec::AbsolutePath(spec.to_owned()))
    } else {
        Ok(PathSpec::RelativePath(spec.to_owned()))
    }
}

pub fn parse_file_source_host_spec(spec: &str) -> Result<HostSpec, FileSourceParseError> {
    if spec == "localhost" {
        return Ok(HostSpec::LocalHost(0));
    }

    let maybe_id = spec.parse::<i32>();
    if let Ok(host_local_id) = maybe_id {
        return Ok(HostSpec::LocalId(host_local_id));
    }

    return Ok(HostSpec::Name(spec.to_owned()));
}
