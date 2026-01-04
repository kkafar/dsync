use dsync_proto::model::server::HostInfo;

use crate::server::{database::models::HostsRow, service::tools};

impl HostsRow {
    pub fn from_host_info(
        host_info: &HostInfo,
        is_remote: bool,
        discovery_time: Option<i64>,
    ) -> Self {
        let discovery_time = discovery_time.unwrap_or_else(|| tools::time::get_current_timestamp());

        HostsRow {
            uuid: host_info.uuid.clone(),
            name: host_info.name.clone(),
            hostname: host_info.hostname.clone(),
            is_remote: is_remote,
            ipv4_addr: host_info.address.clone(),
            discovery_time,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_from_host_info_remote() {
        let host_info = HostInfo {
            uuid: "uuid".to_string(),
            name: "name".to_string(),
            hostname: "hostname".to_string(),
            address: "address".to_string(),
        };

        let host_row = HostsRow::from_host_info(&host_info, true, None);
        assert_common_fields_match_host_info_row(&host_row, &host_info, Some(true), None);
    }

    #[test]
    fn test_from_host_info_local() {
        let host_info = HostInfo {
            uuid: "uuid".to_string(),
            name: "name".to_string(),
            hostname: "hostname".to_string(),
            address: "address".to_string(),
        };

        let host_row = HostsRow::from_host_info(&host_info, false, None);
        assert_common_fields_match_host_info_row(&host_row, &host_info, Some(false), None);
    }

    #[test]
    fn test_from_host_info_discovery_time() {
        let host_info = HostInfo {
            uuid: "uuid".to_string(),
            name: "name".to_string(),
            hostname: "hostname".to_string(),
            address: "address".to_string(),
        };

        let discovery_time: i64 = 1234567890;

        let host_row = HostsRow::from_host_info(&host_info, false, Some(discovery_time));
        assert_common_fields_match_host_info_row(
            &host_row,
            &host_info,
            Some(false),
            Some(discovery_time),
        );
    }

    fn assert_common_fields_match_host_info_row(
        host_row: &HostsRow,
        host_info: &HostInfo,
        is_remote: Option<bool>,
        discovery_time: Option<i64>,
    ) {
        assert_eq!(host_row.uuid, host_info.uuid);
        assert_eq!(host_row.name, host_info.name);
        assert_eq!(host_row.hostname, host_info.hostname);

        if let Some(is_remote) = is_remote {
            assert_eq!(host_row.is_remote, is_remote);
        }

        assert_eq!(host_row.ipv4_addr, host_info.address);
        if let Some(discovery_time) = discovery_time {
            assert_eq!(host_row.discovery_time, discovery_time);
        } else {
            assert!(host_row.discovery_time > 0);
        }
    }
}
