-- Your SQL goes here
create table if not exists local_groups (
    id integer not null primary key,
    name text not null unique
);
