use std::fmt;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CliCommand {
    Help,
    Version,
    Management {
        name: String,
        arguments: Vec<String>,
    },
    Connect {
        alias: String,
        ssh_arguments: Vec<String>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParseError(pub String);

impl fmt::Display for ParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for ParseError {}

/// Parse the user-facing command line after the executable name.
///
/// # Errors
///
/// Returns [`ParseError`] for unknown options or SSH arguments placed before
/// the required `--` separator.
pub fn parse(arguments: impl IntoIterator<Item = String>) -> Result<CliCommand, ParseError> {
    let mut arguments = arguments.into_iter();
    let Some(first) = arguments.next() else {
        return Ok(CliCommand::Help);
    };

    match first.as_str() {
        "-h" | "--help" | "help" => Ok(CliCommand::Help),
        "-V" | "--version" | "version" => Ok(CliCommand::Version),
        "login" | "ls" | "sync" | "status" | "profile" | "agent" | "doctor" | "diagnostics" => {
            Ok(CliCommand::Management {
                name: first,
                arguments: arguments.collect(),
            })
        }
        value if value.starts_with('-') => Err(ParseError(format!("unknown option '{value}'"))),
        alias => {
            let remaining: Vec<String> = arguments.collect();
            let ssh_arguments = match remaining.first().map(String::as_str) {
                None => Vec::new(),
                Some("--") => remaining.into_iter().skip(1).collect(),
                Some(argument) => {
                    return Err(ParseError(format!(
                        "place SSH arguments after '--' (unexpected '{argument}')"
                    )));
                }
            };

            Ok(CliCommand::Connect {
                alias: alias.to_owned(),
                ssh_arguments,
            })
        }
    }
}

pub const HELP: &str = "WarpgateSH — unofficial community client for Warpgate\n\n\
Usage:\n  warpgatesh <target> [-- <ssh arguments>]\n  warpgatesh <command> [arguments]\n\n\
Commands:\n  profile add <name> <url>  Add or replace a Warpgate profile\n  profile list              List configured profiles\n  profile default <name>    Select the profile providing short aliases\n  login <profile>           Replace a personal API token\n  ls                        List synchronized SSH targets\n  sync                      Request an immediate synchronization\n  status                    Show profile and snapshot status\n  agent install             Install and start the background agent\n  agent status              Show whether the background agent is running\n  doctor                    Diagnose the local installation\n  diagnostics preview       Preview local logs before exporting\n  diagnostics export        Create a sanitized ZIP archive in Downloads\n  help                      Show this help\n";

#[must_use]
pub fn openssh_arguments(alias: &str, ssh_arguments: &[String]) -> Vec<String> {
    let mut arguments = Vec::with_capacity(ssh_arguments.len() + 1);
    let destination_index = openssh_destination_index(ssh_arguments);
    arguments.extend_from_slice(&ssh_arguments[..destination_index]);
    arguments.push(alias.to_owned());
    arguments.extend_from_slice(&ssh_arguments[destination_index..]);
    arguments
}

fn openssh_destination_index(arguments: &[String]) -> usize {
    let mut index = 0;
    while index < arguments.len() {
        let argument = &arguments[index];
        if argument == "--" {
            return (index + 1).min(arguments.len());
        }
        if !argument.starts_with('-') || argument == "-" {
            return index;
        }

        index += 1;
        if option_requires_separate_value(argument) && index < arguments.len() {
            index += 1;
        }
    }
    arguments.len()
}

fn option_requires_separate_value(argument: &str) -> bool {
    argument.len() == 2
        && matches!(
            argument.as_bytes()[1],
            b'B' | b'b'
                | b'c'
                | b'D'
                | b'E'
                | b'e'
                | b'F'
                | b'I'
                | b'i'
                | b'J'
                | b'L'
                | b'l'
                | b'm'
                | b'O'
                | b'o'
                | b'P'
                | b'p'
                | b'Q'
                | b'R'
                | b'S'
                | b'W'
                | b'w'
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(ToString::to_string).collect()
    }

    #[test]
    fn treats_an_unknown_word_as_a_target() {
        assert_eq!(
            parse(args(&["dmz-nextcloud-01"])),
            Ok(CliCommand::Connect {
                alias: "dmz-nextcloud-01".to_owned(),
                ssh_arguments: Vec::new(),
            })
        );
    }

    #[test]
    fn forwards_only_arguments_after_the_separator() {
        assert_eq!(
            parse(args(&["dmz-nextcloud-01", "--", "-L", "8080:localhost:80"])),
            Ok(CliCommand::Connect {
                alias: "dmz-nextcloud-01".to_owned(),
                ssh_arguments: args(&["-L", "8080:localhost:80"]),
            })
        );
    }

    #[test]
    fn reserves_management_commands() {
        assert_eq!(
            parse(args(&["sync"])),
            Ok(CliCommand::Management {
                name: "sync".to_owned(),
                arguments: Vec::new(),
            })
        );
    }

    #[test]
    fn puts_openssh_options_before_the_destination() {
        assert_eq!(
            openssh_arguments("dmz-nextcloud-01", &args(&["-L", "8080:localhost:80"])),
            args(&["-L", "8080:localhost:80", "dmz-nextcloud-01"])
        );
    }

    #[test]
    fn puts_a_remote_command_after_the_destination() {
        assert_eq!(
            openssh_arguments("dmz-nextcloud-01", &args(&["true"])),
            args(&["dmz-nextcloud-01", "true"])
        );
    }

    #[test]
    fn separates_openssh_options_from_the_remote_command() {
        assert_eq!(
            openssh_arguments(
                "dmz-nextcloud-01",
                &args(&["-o", "BatchMode=yes", "-p2222", "printf", "connected",]),
            ),
            args(&[
                "-o",
                "BatchMode=yes",
                "-p2222",
                "dmz-nextcloud-01",
                "printf",
                "connected",
            ])
        );
    }
}
