use clap::Subcommand;

#[derive(Subcommand, Debug)]
pub(crate) enum ServerCommand {
    // Request server shutdown
    Shutdown {},

    // Dump running configuration of the server
    PrintConfig {},
}
