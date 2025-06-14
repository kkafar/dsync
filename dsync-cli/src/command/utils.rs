use dsync_proto::{shared, user_agent};
use prettytable::row;

pub(super) fn print_servers_info(server_info_coll: &[shared::ServerInfo]) -> () {
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

pub(super) fn print_local_files_desc(file_descs: &[user_agent::LocalFileDescription]) -> () {
    use prettytable as pt;

    let mut table = pt::Table::new();

    table.add_row(row!["LID", "PATH", "SHA1[:8]"]);

    file_descs.iter().for_each(|desc| {
        table.add_row(row![desc.local_id, &desc.file_path, desc.hash_sha1[0..8]]);
    });

    table.printstd();
}

pub(super) fn print_local_group_info(group_list: &[shared::GroupInfo]) {
    use prettytable as pt;

    let mut table = pt::Table::new();

    table.add_row(row!["LID", "NAME"]);

    group_list.iter().for_each(|group| {
        table.add_row(row![group.local_id, group.name]);
    });

    table.printstd();
}
