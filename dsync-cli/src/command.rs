pub(crate) mod file;
pub(crate) mod group;
pub(crate) mod host;
pub(crate) mod model;

use std::path::PathBuf;

use anyhow::Context;
use dsync_proto::cli::{
    self, AddFileRequest, DiscoverHostsRequest, ListHostsRequest, ListLocalFilesRequest,
    client_api_client::ClientApiClient,
};
use prettytable::row;

use crate::cli::command::Commands;

const LOOPBACK_ADDR_V4: &str = "http://127.0.0.1:50051";

impl Commands {
    pub(super) async fn handle_list_hosts(self) -> anyhow::Result<()> {
        let mut client = ClientApiClient::connect(LOOPBACK_ADDR_V4).await?;

        let request = tonic::Request::new(ListHostsRequest {});

        log::info!("Sending request to server");
        log::debug!("{request:?}");

        let response = client.list_hosts(request).await?;

        log::info!("Received response from server");
        log::debug!("{response:?}");

        let response_payload = response.into_inner();
        print_servers_info(&response_payload.servers_info);

        anyhow::Ok(())
    }

    pub(super) async fn handle_discover_hosts(self) -> anyhow::Result<()> {
        let mut client = ClientApiClient::connect(LOOPBACK_ADDR_V4).await?;

        let request = tonic::Request::new(DiscoverHostsRequest {});

        log::info!("Sending request to server");
        log::debug!("{request:?}");

        let response = client.discover_hosts(request).await?;

        log::info!("Received response from server");
        log::debug!("{response:?}");

        let response_payload = response.into_inner();
        print_servers_info(&response_payload.servers_info);

        anyhow::Ok(())
    }

    // This captures self by reference to avoid problems with dispatching,
    // since the command itself owns the file_path.
    pub(super) async fn handle_add_file(&self, file_path: PathBuf) -> anyhow::Result<()> {
        let file_path_abs = match file_path.canonicalize() {
            Ok(abs_path) => abs_path,
            Err(err) => {
                let message = format!(
                    "Failed to turn file_path: ${file_path:?} into absolute path with err: {err}"
                );
                log::error!("{message}");
                return Err(anyhow::anyhow!(message));
            }
        };

        let path_as_string = file_path_abs
            .to_str()
            .context("Looks like the specified path is not a valid unicode")?
            .to_string();

        let file_paths: Vec<String> = vec![path_as_string];
        let request = tonic::Request::new(AddFileRequest {
            file_path: file_paths,
        });

        let mut client = ClientApiClient::connect(LOOPBACK_ADDR_V4).await?;

        log::info!("Sending request to server");
        log::debug!("{request:?}");

        let response = client.add_file(request).await?;

        log::info!("Received response from server");
        log::debug!("{response:?}");

        anyhow::Ok(())
    }

    pub(super) async fn handle_list_local_files(&self) -> anyhow::Result<()> {
        let request = tonic::Request::new(ListLocalFilesRequest {});
        let mut client = ClientApiClient::connect(LOOPBACK_ADDR_V4).await?;

        log::info!("Sending request to server");
        log::debug!("{request:?}");

        let response = client.list_local_files(request).await?;

        log::info!("Received response from server");
        log::debug!("{response:?}");

        let payload = response.into_inner();

        print_local_files_desc(&payload.file_descs);

        anyhow::Ok(())
    }
}

fn print_servers_info(server_info_coll: &[cli::ServerInfo]) -> () {
    use prettytable as pt;

    let mut table = pt::Table::new();
    table.add_row(row!["LID", "NAME", "HOSTNAME", "ADDR"]);

    server_info_coll
        .into_iter()
        .enumerate()
        .for_each(|(i, info)| {
            table.add_row(row![i, info.name, info.hostname, info.address]);
        });

    table.printstd();
}

fn print_local_files_desc(file_descs: &[cli::LocalFileDescription]) -> () {
    use prettytable as pt;

    let mut table = pt::Table::new();

    table.add_row(row!["LID", "PATH", "SHA1[:8]"]);

    file_descs.iter().for_each(|desc| {
        table.add_row(row![desc.local_id, &desc.file_path, desc.hash_sha1[0..8]]);
    });

    table.printstd();
}
