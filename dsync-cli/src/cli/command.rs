mod command_impl;

use std::path::PathBuf;

use clap::Subcommand;

#[derive(Subcommand)]
pub(crate) enum Commands {
    ListHosts,
    DiscoverHosts,
    AddFile { file_path: PathBuf },
}

impl Commands {
    pub(crate) async fn handle(self) -> anyhow::Result<()> {
        match self {
            Self::ListHosts => self.handle_list_hosts().await,
            Self::DiscoverHosts => self.handle_discover_hosts().await,
            Self::AddFile { ref file_path } => self.handle_add_file(file_path.clone()).await,
        }
    }
}
