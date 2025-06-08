use clap::Subcommand;

#[derive(Subcommand)]
pub(crate) enum HostCommand {
    List {
        /// Run host discovery before returning the result
        #[arg(short = 'd', long = "discover")]
        discover: Option<bool>,
    },
    Discover,
}
