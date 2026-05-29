use clap::{Arg, Command};

#[allow(dead_code)]
pub fn create_test_command() -> Command {
    Command::new("test-cli")
        .version("1.0.0")
        .about("A test CLI for verification")
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .help("Enable verbose logging")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("port")
                .long("port")
                .help("Port number")
                .num_args(1)
                .action(clap::ArgAction::Set),
        )
        .arg(Arg::new("tags").long("tag").help("Tags list").action(clap::ArgAction::Append))
        .subcommand(
            Command::new("start").about("Start the service").arg(
                Arg::new("daemon")
                    .long("daemon")
                    .help("Run in daemon mode")
                    .action(clap::ArgAction::SetTrue),
            ),
        )
}
