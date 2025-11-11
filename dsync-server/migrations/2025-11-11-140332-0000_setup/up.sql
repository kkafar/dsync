-- Your SQL goes herE

create table if not exists hosts (
    uuid text not null unique primary key,
    name text not null unique,
    hostname text not null unique,
    type text not null, -- 'local' or 'remote'
    ipv4_addr text not null,
    discovery_time bigint not null
);

create table if not exists files_local (
    id integer not null primary key,
    file_path text not null unique,
    hash_sha1 text not null unique
);

create table if not exists files_tracked (
    local_id integer primary key,
    peer_uuid text not null,
    remote_id integer not null unique
);

create table if not exists groups_local (
    id integer not null primary key,
    name text not null unique
);

create table if not exists group_files_local (
    file_id integer not null,
    group_id integer not null,
    primary key (file_id, group_id),
    foreign key (file_id) references files_local (
        id
    ) on update cascade on delete cascade,
    foreign key (group_id) references groups_local (
        id
    ) on update cascade on delete cascade
);
