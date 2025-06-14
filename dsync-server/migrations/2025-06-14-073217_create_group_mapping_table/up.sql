-- Your SQL goes here
create table if not exists local_file_grouping (
    file_id integer not null,
    group_id integer not null,
    primary key (file_id, group_id),
    foreign key (file_id) references local_files (id) on update cascade on delete cascade,
    foreign key (group_id) references local_groups (id) on update cascade on delete cascade
);
