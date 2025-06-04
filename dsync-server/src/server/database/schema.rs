// @generated automatically by Diesel CLI.

diesel::table! {
    local_files (id) {
        id -> Integer,
        file_path -> Text,
        hash_sha1 -> Text,
    }
}

diesel::table! {
    local_server_base_info (uuid) {
        uuid -> Text,
        name -> Text,
        hostname -> Text,
    }
}

diesel::table! {
    peer_addr_v4 (uuid) {
        uuid -> Text,
        ipv4_addr -> Text,
        discovery_time -> BigInt,
    }
}

diesel::table! {
    peer_server_base_info (uuid) {
        uuid -> Text,
        name -> Text,
        hostname -> Text,
    }
}

diesel::table! {
    synced_files (local_id) {
        local_id -> Nullable<Integer>,
        peer_uuid -> Text,
        remote_file_id -> Integer,
    }
}

diesel::joinable!(peer_addr_v4 -> peer_server_base_info (uuid));

diesel::allow_tables_to_appear_in_same_query!(
    local_files,
    local_server_base_info,
    peer_addr_v4,
    peer_server_base_info,
    synced_files,
);
