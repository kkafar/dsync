use dsync_proto::model::{
    common::LocalFileDescription,
    server::{GroupInfo, HostInfo},
};
use prettytable::row;

pub(super) fn print_servers_info(server_info_coll: &[HostInfo]) {
    use prettytable as pt;

    let mut table = pt::Table::new();
    table.add_row(row!["LID", "NAME", "HOSTNAME", "ADDR"]);

    server_info_coll.iter().enumerate().for_each(|(i, info)| {
        table.add_row(row![i, info.name, info.hostname, info.address]);
    });

    table.printstd();
}

pub(super) fn print_local_files_desc(file_descs: &[LocalFileDescription]) {
    use prettytable as pt;

    let mut table = pt::Table::new();

    table.add_row(row!["LID", "PATH", "SHA1[:8]"]);

    file_descs.iter().for_each(|desc| {
        table.add_row(row![desc.local_id, &desc.file_path, desc.hash_sha1[0..8]]);
    });

    table.printstd();
}

pub(super) fn print_local_group_info(group_list: &[GroupInfo]) {
    use prettytable as pt;

    let mut table = pt::Table::new();

    table.add_row(row!["LID", "NAME"]);

    group_list.iter().for_each(|group| {
        table.add_row(row![group.local_id, group.name]);
    });

    table.printstd();
}
