//! This package contains code generated from protobuf specs,
//! shared between client & server implementations.

pub mod cli {
    include!("../proto-generated/cli.rs");
}

pub mod p2p {
    include!("../proto-generated/server.rs");
}
