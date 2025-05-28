create table if not exists this_server_info (
    uuid text not null unique primary key,
    name text not null unique,
    hostname text not null unique
);

create table if not exists server_info (
    uuid text not null unique primary key,
    name text not null,
    hostname text not null,
    address text not null
);
