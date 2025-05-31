use diesel::prelude::*;

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::server_info)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct ServerInfoRow {
    pub uuid: String,
    pub name: String,
    pub hostname: String,
    pub address: String,
}

#[derive(Queryable, Selectable, Insertable, Clone)]
#[diesel(table_name = crate::schema::this_server_info)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct ThisServerInfoRow {
    pub uuid: String,
    pub name: String,
    pub hostname: String,
}
