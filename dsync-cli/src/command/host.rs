use std::{net::Ipv4Addr, str::FromStr};

use dsync_proto::services::user_agent::{
    HostAddRequest, HostDiscoverRequest, HostListRequest, HostRemoveRequest,
};
use dsync_shared::{conn::ServiceConnFactory, model::parse_file_source_host_spec};

use crate::command::utils;

pub(crate) async fn host_list() -> anyhow::Result<()> {
    let mut client = ServiceConnFactory::local_user_agent_service(None).await?;

    let request = tonic::Request::new(HostListRequest { discover: false });

    log::info!("Sending request to server");
    log::debug!("{request:?}");

    let response = client.host_list(request).await?;

    log::info!("Received response from server");
    log::debug!("{response:?}");

    let response_payload = response.into_inner();
    utils::print_servers_info(&response_payload.host_list);

    anyhow::Ok(())
}

pub(crate) async fn host_discover() -> anyhow::Result<()> {
    let mut client = ServiceConnFactory::local_user_agent_service(None).await?;

    let request = tonic::Request::new(HostDiscoverRequest {});

    log::info!("Sending request to server");
    log::debug!("{request:?}");

    let response = client.host_discover(request).await?;

    log::info!("Received response from server");
    log::debug!("{response:?}");
    let response_payload = response.into_inner();
    utils::print_servers_info(&response_payload.host_list);

    anyhow::Ok(())
}

pub(crate) async fn host_add(host_addr: String) -> Result<(), anyhow::Error> {
    let mut client = ServiceConnFactory::local_user_agent_service(None).await?;

    let host_addr_spec = parse_host_addr_spec(&host_addr)?;

    let ipv4_addr_str: String = host_addr_spec.0.to_string();
    let port: Option<u32> = host_addr_spec.1.map(Into::<u32>::into);

    let request = tonic::Request::new(HostAddRequest {
        ipv4_addr: ipv4_addr_str,
        port,
    });

    log::info!("Sending request to server");
    log::debug!("{request:?}");

    let response = client.host_add(request).await?;

    log::info!("Received response from server");
    log::debug!("{response:?}");
    let response_payload = response.into_inner();
    println!(
        "Host added successfuly: {:?}",
        response_payload.host_info.expect("Payload is required")
    );

    Ok(())
}

pub(crate) async fn host_remove(host_spec: String) -> Result<(), anyhow::Error> {
    let mut client = ServiceConnFactory::local_user_agent_service(None).await?;

    let host_spec = parse_file_source_host_spec(&host_spec)?;

    let request = tonic::Request::new(HostRemoveRequest {
        host_spec: Some(host_spec),
    });

    log::info!("Sending request to server");
    log::debug!("{request:?}");

    let _ = client.host_remove(request).await?;
    println!("Host removed successfuly");

    Ok(())
}

fn parse_host_addr_spec(spec: impl AsRef<str>) -> Result<(Ipv4Addr, Option<u16>), anyhow::Error> {
    let spec = spec.as_ref();

    if !spec.contains(":") {
        return Ok((Ipv4Addr::from_str(spec)?, None));
    }

    let mut split_str = spec.splitn(2, ":");
    let addr_part = split_str.next().ok_or(anyhow::anyhow!(
        "Invalid addr format - failed to parse addr"
    ))?;
    let port_part = split_str.next().ok_or(anyhow::anyhow!(
        "Invalid addr format - failed to parse port"
    ))?;

    let ipv4_addr = Ipv4Addr::from_str(addr_part)?;
    let port = u16::from_str(port_part)?;

    Ok((ipv4_addr, Some(port)))
}
