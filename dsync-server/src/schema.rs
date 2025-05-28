// @generated automatically by Diesel CLI.

diesel::table! {
    server_info (uuid) {
        uuid -> Text,
        name -> Text,
        hostname -> Text,
        address -> Text,
    }
}

diesel::table! {
    this_server_info (uuid) {
        uuid -> Text,
        name -> Text,
        hostname -> Text,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    server_info,
    this_server_info,
);
