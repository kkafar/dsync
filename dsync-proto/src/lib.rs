//! This package contains code generated from protobuf specs,
//! shared between client & server implementations.

pub mod services {
    pub mod file_transfer {
        include!("../proto-generated/services.file_transfer.rs");
    }

    pub mod host_discovery {
        include!("../proto-generated/services.host_discovery.rs");
    }

    pub mod user_agent {
        include!("../proto-generated/services.user_agent.rs");
    }
}

pub mod model {
    pub mod common {
        include!("../proto-generated/model.common.rs");
    }

    pub mod server {
        include!("../proto-generated/model.server.rs");
    }
}

// pub mod shared {
//     include!("../proto-generated/shared.rs");
// }
//
// pub mod user_agent {
//     include!("../proto-generated/user_agent.rs");
// }
//
// pub mod server {
//     include!("../proto-generated/server.rs");
// }
//
// pub mod file_transfer {
//     include!("../proto-generated/filetransfer.rs");
// }
