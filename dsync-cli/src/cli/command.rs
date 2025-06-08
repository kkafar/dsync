mod cmdimpl;
pub(crate) mod model;

impl model::Commands {
    pub(crate) async fn handle(self) -> anyhow::Result<()> {
        match self {
            Self::ListHosts => self.handle_list_hosts().await,
            Self::DiscoverHosts => self.handle_discover_hosts().await,
            Self::AddFile { ref file_path } => self.handle_add_file(file_path.clone()).await,
            Self::ListLocalFiles => self.handle_list_local_files().await,
            _ => {
                log::warn!("Unimplemented command!");
                Err(anyhow::anyhow!("Unimplemented command"))
            }
        }
    }
}
