// @generated automatically by Diesel CLI.

diesel::table! {
    files_local (id) {
        id -> Integer,
        file_path -> Text,
        hash_sha1 -> Text,
    }
}

diesel::table! {
    files_tracked (local_id) {
        local_id -> Nullable<Integer>,
        peer_uuid -> Text,
        remote_id -> Integer,
    }
}

diesel::table! {
    group_files_local (file_id, group_id) {
        file_id -> Integer,
        group_id -> Integer,
    }
}

diesel::table! {
    groups_local (id) {
        id -> Integer,
        name -> Text,
    }
}

diesel::table! {
    hosts (uuid) {
        uuid -> Text,
        name -> Text,
        hostname -> Text,
        #[sql_name = "type"]
        type_ -> Text,
        ipv4_addr -> Text,
        discovery_time -> BigInt,
    }
}

diesel::joinable!(group_files_local -> files_local (file_id));
diesel::joinable!(group_files_local -> groups_local (group_id));

diesel::allow_tables_to_appear_in_same_query!(
    files_local,
    files_tracked,
    group_files_local,
    groups_local,
    hosts,
);
