use clap::Subcommand;

#[derive(Subcommand, Debug)]
pub(crate) enum HostCommand {
    /// List all host known by local deamon.
    List {
        /// Run host discovery before returning the result
        #[arg(short = 'd', long = "discover")]
        discover: Option<bool>,
    },
    /// Try to discover hosts on local network & then return the list of known hosts.
    Discover,

    /// Manually add a host to the known hosts list. The operation
    /// will succeed only in case the host is reachable when calling it.
    Add {
        /// Address to try the host discovery at. Accepted syntax: IPV4[:PORT].
        /// If port is not specified - default will be used.
        ipv4_addr: String,
    },

    /// Manually remove a host from the known hosts list.
    Remove {
        /// This might be either the local host id or name.
        host_spec: String,
    },
}
