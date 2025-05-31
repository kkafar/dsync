create table if not exists local_server_base_info (
    uuid text not null unique primary key,
    name text not null unique,
    hostname text not null unique
);

create table if not exists peer_server_base_info (
    uuid text not null unique primary key,
    name text not null,
    hostname text not null,
    address text not null
);

create table if not exists peer_addr_v4 (
    uuid text not null unique primary key,
    ipv4_addr text not null,
    discovery_time bigint not null,
    foreign key (uuid) references peer_server_base_info (uuid) on update cascade on delete cascade
);
