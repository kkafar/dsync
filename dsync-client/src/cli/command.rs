mod command_impl;

use clap::Subcommand;

#[derive(Subcommand)]
pub(crate) enum Commands {
    ListHosts,
}

impl Commands {
    pub(crate) async fn handle(self) -> anyhow::Result<()> {
        match self {
            Self::ListHosts => self.handle_list_hosts().await,
        }
    }
}
