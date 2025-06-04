create table if not exists synced_files (
    local_id integer primary key,
    peer_uuid text not null,
    remote_file_id integer not null unique
);
