use std::process::{Command, ExitCode};

use warpgatesh_cli::{CliCommand, HELP, openssh_arguments, parse};

fn main() -> ExitCode {
    let command = match parse(std::env::args().skip(1)) {
        Ok(command) => command,
        Err(error) => {
            eprintln!("warpgatesh: {error}");
            eprintln!("Try 'warpgatesh help' for usage.");
            return ExitCode::from(2);
        }
    };

    match command {
        CliCommand::Help => {
            print!("{HELP}");
            ExitCode::SUCCESS
        }
        CliCommand::Version => {
            println!("warpgatesh {}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        CliCommand::Management { name, .. } => {
            eprintln!("warpgatesh: '{name}' is part of the next development increment");
            ExitCode::from(2)
        }
        CliCommand::Connect {
            alias,
            ssh_arguments,
        } => execute_ssh(&alias, &ssh_arguments),
    }
}

#[cfg(unix)]
fn execute_ssh(alias: &str, ssh_arguments: &[String]) -> ExitCode {
    use std::os::unix::process::CommandExt;

    let error = Command::new("/usr/bin/ssh")
        .args(openssh_arguments(alias, ssh_arguments))
        .exec();
    eprintln!("warpgatesh: could not execute /usr/bin/ssh: {error}");
    ExitCode::FAILURE
}

#[cfg(not(unix))]
fn execute_ssh(_alias: &str, _ssh_arguments: &[String]) -> ExitCode {
    eprintln!("warpgatesh: OpenSSH delegation is not supported on this platform");
    ExitCode::FAILURE
}
