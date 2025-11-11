//! This package contains code generated from protobuf specs,
//! shared between client & server implementations.

pub mod shared {
    include!("../proto-generated/shared.rs");
}

pub mod user_agent {
    include!("../proto-generated/user_agent.rs");
}

pub mod server {
    include!("../proto-generated/server.rs");
}

pub mod file_transfer {
    include!("../proto-generated/filetransfer.rs");
}
