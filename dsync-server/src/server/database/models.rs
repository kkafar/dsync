use diesel::prelude::*;

#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = super::schema::peer_server_base_info)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct PeerServerBaseInfoRow {
    pub uuid: String,
    pub name: String,
    pub hostname: String,
}

#[derive(Queryable, Selectable, Insertable, Clone, Debug)]
#[diesel(table_name = super::schema::local_server_base_info)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct LocalServerBaseInfoRow {
    pub uuid: String,
    pub name: String,
    pub hostname: String,
}

#[derive(Queryable, Selectable, Insertable, Clone)]
#[diesel(table_name = super::schema::peer_addr_v4)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct PeerAddrV4Row {
    pub uuid: String,
    pub ipv4_addr: String,
    pub discovery_time: i64,
}

#[derive(Queryable, Selectable, Insertable, Clone, Debug)]
#[diesel(table_name = super::schema::local_files)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct LocalFilesWoIdRow {
    pub file_path: String,
    pub hash_sha1: String,
}

#[derive(Queryable, Selectable, Insertable, Clone, Debug)]
#[diesel(table_name = super::schema::local_files)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct LocalFilesRow {
    pub id: i32,
    pub file_path: String,
    pub hash_sha1: String,
}
