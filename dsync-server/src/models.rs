use diesel::prelude::*;

#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::peer_server_base_info)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct PeerServerBaseInfoRow {
    pub uuid: String,
    pub name: String,
    pub hostname: String,
}

#[derive(Queryable, Selectable, Insertable, Clone)]
#[diesel(table_name = crate::schema::local_server_base_info)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct LocalServerBaseInfoRow {
    pub uuid: String,
    pub name: String,
    pub hostname: String,
}

#[derive(Queryable, Selectable, Insertable, Clone)]
#[diesel(table_name = crate::schema::peer_addr_v4)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct PeerAddrV4Row {
    pub uuid: String,
    pub ipv4_addr: String,
    pub discovery_time: i64,
}
