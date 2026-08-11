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
        "login" | "ls" | "sync" | "status" | "profile" | "agent" | "doctor" => {
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
Commands:\n  profile add <name> <url>  Add or replace a Warpgate profile\n  profile list              List configured profiles\n  profile default <name>    Select the profile providing short aliases\n  login <profile>           Replace a personal API token\n  ls                        List synchronized SSH targets\n  sync                      Request an immediate synchronization\n  status                    Show profile and snapshot status\n  doctor                    Diagnose the local installation\n  help                      Show this help\n";

#[must_use]
pub fn openssh_arguments(alias: &str, ssh_arguments: &[String]) -> Vec<String> {
    let mut arguments = Vec::with_capacity(ssh_arguments.len() + 1);
    arguments.extend_from_slice(ssh_arguments);
    arguments.push(alias.to_owned());
    arguments
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
}
