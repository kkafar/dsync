create table if not exists local_files (
    id integer not null primary key,
    file_path text not null unique,
    hash_sha1 text not null unique
);
