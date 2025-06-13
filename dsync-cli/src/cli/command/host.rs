use clap::Subcommand;

#[derive(Subcommand, Debug)]
pub(crate) enum HostCommand {
    /// List all host known by local deamon.
    List {
        /// Run host discovery before returning the result
        #[arg(short = 'd', long = "discover")]
        discover: Option<bool>,
    },
    /// First try to discover hosts on local network & then return the list of known hosts.
    Discover,
}
