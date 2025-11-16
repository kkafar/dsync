use std::{net::Ipv4Addr, str::FromStr};

use serde::Deserialize;

pub(crate) fn discover_hosts_in_local_network() -> Option<Vec<Ipv4Addr>> {
    let nmap_result = std::process::Command::new("nmap")
        .arg("-sP")
        .arg("192.168.100.1/24")
        .status();

    if let Err(err) = nmap_result {
        log::error!("nmap run failed with error: {err}");
        return None;
    }

    // Lines for existing & up hosts.
    let ipv4_list: Vec<Ipv4Addr> = CandidateAddressProviderFactory::new()
        .make_provider()
        .unwrap()
        .compute_list()
        .unwrap();

    return Some(ipv4_list);
}

pub trait CandidateAddressProvider {
    fn compute_list(&self) -> anyhow::Result<Vec<Ipv4Addr>>;
}

#[derive(Debug, Clone)]
pub struct ArpAddressProvider;

impl CandidateAddressProvider for ArpAddressProvider {
    fn compute_list(&self) -> anyhow::Result<Vec<Ipv4Addr>> {
        let arp_result = std::process::Command::new("arp").arg("-a").output();

        if let Err(err) = arp_result {
            anyhow::bail!("Failed to run arp: {err}");
        }

        let Ok(arp_output) = arp_result else {
            anyhow::bail!("Failed to retrieve output of arp");
        };

        let output_string: String = match String::from_utf8(arp_output.stdout) {
            Ok(output_string) => output_string,
            Err(err) => {
                anyhow::bail!("Failed to convert arp output to string! Err: {err}");
            }
        };

        // Lines for existing & up hosts.
        let ipv4_list: Vec<Ipv4Addr> = output_string
            .lines()
            .filter(|&line| {
                return !line.contains("incomplete") && !line.contains("_gateway");
            })
            .filter_map(|line| {
                let Some(left_brace_index) = line.find('(') else {
                    log::error!("Unexpected format of arp output line: {line}");
                    return None;
                };

                let Some(right_brace_index) = line.find(')') else {
                    log::error!("Unexpected format of arp output line: {line}");
                    return None;
                };

                assert!(left_brace_index < right_brace_index);

                let ip_slice = &line[left_brace_index + 1..right_brace_index];

                assert!(!ip_slice.is_empty());
                return Some(ip_slice);
            })
            .map(|ip_slice| {
                Ipv4Addr::from_str(ip_slice).inspect_err(|err| {
                    log::error!("Failed to parse addr with err: {err}");
                })
            })
            .filter_map(Result::ok)
            .collect();

        return Ok(ipv4_list);
    }
}

#[derive(Debug, Clone)]
pub struct IpNeighAddressProvider;

impl CandidateAddressProvider for IpNeighAddressProvider {
    fn compute_list(&self) -> anyhow::Result<Vec<Ipv4Addr>> {
        let cmd_output = match std::process::Command::new("ip")
            .arg("-json")
            .arg("neighbour")
            .output()
        {
            Ok(result) => result,
            Err(err) => {
                anyhow::bail!("Failed to run `ip neighbour` command, err: {err}");
            }
        };

        let output_string: String = match String::from_utf8(cmd_output.stdout) {
            Ok(output_string) => output_string,
            Err(err) => {
                anyhow::bail!("Failed to convert arp output to string! Err: {err}");
            }
        };

        let neigh_objects: Vec<IpNeighObject> = serde_json::from_str(&output_string)?;

        let ipv4_list = neigh_objects
            .into_iter()
            .filter(|neigh| {
                return !neigh.dst.contains(":")
                    && !["FAILED", "INCOMPLETE"].contains(&neigh.state.first().unwrap().as_str());
            })
            .map(|neigh| Ipv4Addr::from_str(&neigh.dst))
            .filter_map(Result::ok)
            .collect::<Vec<Ipv4Addr>>();

        Ok(ipv4_list)
    }
}

pub(crate) struct CandidateAddressProviderFactory;

impl CandidateAddressProviderFactory {
    pub fn new() -> Self {
        Self
    }

    pub fn make_provider(&self) -> Option<Box<dyn CandidateAddressProvider>> {
        if let Some(arp_provider) = self.make_arp_provider() {
            log::debug!("ARP PROVIDER");
            return Some(arp_provider);
        }

        if let Some(ip_neigh_provider) = self.make_ip_neigh_provider() {
            log::debug!("IP NEIGH PROVIDER");
            return Some(ip_neigh_provider);
        }

        return None;
    }

    fn make_arp_provider(&self) -> Option<Box<dyn CandidateAddressProvider>> {
        log::debug!("MAKE ARP BEFORE CHECK");
        if !super::super::file::check_binary_exists("arp") {
            log::debug!("MAKE ARP AFTER CHECK");
            return None;
        }

        return Some(Box::new(ArpAddressProvider));
    }

    fn make_ip_neigh_provider(&self) -> Option<Box<dyn CandidateAddressProvider>> {
        if !super::super::file::check_binary_exists("ip") {
            return None;
        }

        return Some(Box::new(IpNeighAddressProvider));
    }
}

#[derive(Deserialize)]
struct IpNeighObject {
    pub dst: String,
    #[allow(unused)]
    pub dev: String,
    pub state: Vec<String>,
}
