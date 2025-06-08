//! This package contains code generated from protobuf specs,
//! shared between client & server implementations.

pub mod shared {
    include!("../proto-generated/shared.rs");
}

pub mod cli {
    include!("../proto-generated/cli.rs");
}

pub mod server {
    include!("../proto-generated/server.rs");
}
