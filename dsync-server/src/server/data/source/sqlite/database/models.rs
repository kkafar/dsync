use diesel::prelude::*;

#[derive(Debug, Clone, Queryable, Selectable, Insertable)]
#[diesel(table_name = super::schema::files_local)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct FilesLocalRow {
    pub id: i32,
    pub file_path: String,
    pub hash_sha1: String,
}

#[derive(Debug, Clone, Queryable, Selectable, Insertable)]
#[diesel(table_name = super::schema::files_local)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct FilesLocalFragmentInsert {
    pub file_path: String,
    pub hash_sha1: String,
}

#[derive(Debug, Clone, Queryable, Selectable, Insertable)]
#[diesel(table_name = super::schema::files_tracked)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct FilesTrackedRow {
    pub local_id: i32,
    pub peer_uuid: String,
    pub remote_id: i32,
}

#[derive(Debug, Clone, Queryable, Selectable, Insertable)]
#[diesel(table_name = super::schema::groups_local)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct GroupsLocalRow {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Clone, Queryable, Selectable, Insertable)]
#[diesel(table_name = super::schema::groups_local)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct GroupsLocalFragmentInsert {
    pub name: String,
}

#[derive(Debug, Clone, Queryable, Selectable, Insertable)]
#[diesel(table_name = super::schema::group_files_local)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct GroupFilesLocalRow {
    pub file_id: i32,
    pub group_id: i32,
}

#[derive(Debug, Clone, Queryable, Selectable, Insertable)]
#[diesel(table_name = super::schema::hosts)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct HostsRow {
    pub uuid: String,
    pub name: String,
    pub hostname: String,
    pub is_remote: bool,
    pub ipv4_addr: String,
    pub discovery_time: i64,
}
