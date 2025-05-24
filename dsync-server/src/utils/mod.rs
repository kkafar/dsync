use std::{net::Ipv4Addr, str::FromStr};

/// Basically tries to call `which ${binary_name}` & reports the command status.
/// Returns false if the check has failed for some other reason!
pub(crate) fn check_binary_exists(binary_name: &str) -> bool {
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

    if exit_status.success() {
        return true;
    } else {
        return false;
    }
}

pub(crate) fn discover_hosts_in_local_network(require_nmap_pass: bool) -> Option<Vec<Ipv4Addr>> {
    let nmap_result = std::process::Command::new("nmap")
        .arg("-sP")
        .arg("192.168.100.1/24")
        .status();

    if let Err(err) = nmap_result {
        println!("nmap run failed with error: {err}");
        return None;
    }

    let arp_result = std::process::Command::new("arp").arg("-a").output();

    if let Err(err) = arp_result {
        println!("arp run failed with error: {err}");
        return None;
    }

    let Ok(arp_output) = arp_result else {
        panic!("Something went seriously wrong at this point");
    };

    let output_string: String = match String::from_utf8(arp_output.stdout) {
        Ok(output_string) => output_string,
        Err(err) => {
            println!("Failed to convert arp output to string! Err: {err}");
            return None;
        }
    };

    // Lines for existing & up hosts.
    let ipv4_list: Vec<Ipv4Addr> = output_string
        .lines()
        .filter(|&line| {
            println!("Parsing line: {line}");
            return !line.contains("<incomplete>") && !line.contains("_gateway");
        })
        .filter_map(|line| {
            println!("Handling line: {line}");
            let Some(left_brace_index) = line.find('(') else {
                println!("Unexpected format of arp output line: {line}");
                return None;
            };

            let Some(right_brace_index) = line.find(')') else {
                println!("Unexpected format of arp output line: {line}");
                return None;
            };

            assert!(left_brace_index < right_brace_index);

            let ip_slice = &line[left_brace_index + 1..right_brace_index];

            assert!(!ip_slice.is_empty());
            return Some(ip_slice);
        })
        .map(|ip_slice| {
            Ipv4Addr::from_str(ip_slice).inspect_err(|err| {
                println!("Failed to parse addr with err: {err}");
            })
        })
        .filter_map(Result::ok)
        .collect();

    return Some(ipv4_list);
}
