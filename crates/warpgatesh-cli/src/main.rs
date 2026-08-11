use std::io::{self, Write};
use std::path::Path;
use std::process::{Command, ExitCode};
use std::time::{SystemTime, UNIX_EPOCH};

use warpgatesh_cli::{CliCommand, HELP, openssh_arguments, parse};
use warpgatesh_core::aliases::is_valid_profile_name;
use warpgatesh_core::profiles::Profile;
use warpgatesh_runtime::RuntimeError;
use warpgatesh_runtime::api::ApiClient;
use warpgatesh_runtime::keychain::{SystemKeychain, TokenStore};
use warpgatesh_runtime::ssh::{
    install_managed_include, open_token_page, save_host_keys, scan_host_keys,
};
use warpgatesh_runtime::storage::LocalStore;

fn main() -> ExitCode {
    let command = match parse(std::env::args().skip(1)) {
        Ok(command) => command,
        Err(error) => {
            eprintln!("warpgatesh: {error}");
            eprintln!("Try 'warpgatesh help' for usage.");
            return ExitCode::from(2);
        }
    };

    match run(command) {
        Ok(code) => code,
        Err(error) => {
            eprintln!("warpgatesh: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run(command: CliCommand) -> Result<ExitCode, RuntimeError> {
    match command {
        CliCommand::Help => {
            print!("{HELP}");
            Ok(ExitCode::SUCCESS)
        }
        CliCommand::Version => {
            println!("warpgatesh {}", env!("CARGO_PKG_VERSION"));
            Ok(ExitCode::SUCCESS)
        }
        CliCommand::Management { name, arguments } => {
            run_management(&name, &arguments)?;
            Ok(ExitCode::SUCCESS)
        }
        CliCommand::Connect {
            alias,
            ssh_arguments,
        } => {
            warn_if_snapshot_is_stale();
            Ok(execute_ssh(&alias, &ssh_arguments))
        }
    }
}

fn run_management(name: &str, arguments: &[String]) -> Result<(), RuntimeError> {
    match name {
        "profile" => run_profile(arguments),
        "login" => login(arguments),
        "ls" => list_targets(arguments),
        "sync" => {
            require_no_arguments("sync", arguments)?;
            run_agent_once()
        }
        "status" => status(arguments),
        "doctor" => doctor(arguments),
        "agent" => Err(RuntimeError::InvalidInput(
            "agent management will be added with the persistent LaunchAgent".to_owned(),
        )),
        _ => Err(RuntimeError::InvalidInput(format!(
            "unknown management command '{name}'"
        ))),
    }
}

fn run_profile(arguments: &[String]) -> Result<(), RuntimeError> {
    match arguments {
        [command, name, url] if command == "add" => add_profile(name, url),
        [command] if command == "list" => list_profiles(),
        [command, name] if command == "default" => set_default_profile(name),
        _ => Err(RuntimeError::InvalidInput(
            "usage: warpgatesh profile add <name> <url> | profile list | profile default <name>"
                .to_owned(),
        )),
    }
}

fn add_profile(name: &str, url: &str) -> Result<(), RuntimeError> {
    if !is_valid_profile_name(name) {
        return Err(RuntimeError::InvalidInput(format!(
            "invalid profile name '{name}'; use lowercase letters, digits, and hyphens"
        )));
    }
    let client = ApiClient::new(url)?;
    let token_page = client.token_page_url()?;
    println!("Opening {token_page}");
    if let Err(error) = open_token_page(token_page.as_str()) {
        eprintln!("warpgatesh: {error}; open the URL above manually");
    }

    let token = prompt_token()?;
    let metadata = client.validate(&token)?;
    println!(
        "Authenticated as {} on Warpgate {}",
        metadata.username,
        metadata.version.as_deref().unwrap_or("version unknown")
    );

    println!(
        "Retrieving SSH host keys from {}:{}…",
        metadata.ssh_host, metadata.ssh_port
    );
    let host_keys = scan_host_keys(&metadata.ssh_host, metadata.ssh_port)?;
    println!("\nSSH host-key fingerprints:\n{}", host_keys.fingerprints);
    if !confirm("Trust and pin these SSH host keys? [y/N] ")? {
        return Err(RuntimeError::InvalidInput(
            "profile creation cancelled; no data was changed".to_owned(),
        ));
    }

    let store = LocalStore::for_current_user()?;
    let mut catalog = store.load_profiles()?;
    catalog.upsert(Profile {
        name: name.to_owned(),
        base_url: client.base_url().as_str().to_owned(),
        username: metadata.username,
        warpgate_version: metadata.version,
        ssh_host: metadata.ssh_host,
        ssh_port: metadata.ssh_port,
    })?;

    SystemKeychain.set(name, &token)?;
    save_host_keys(store.paths(), name, &host_keys.known_hosts)?;
    store.save_profiles(&catalog)?;
    let include_added = install_managed_include(store.paths())?;
    println!(
        "Profile '{name}' saved{}.",
        if include_added {
            " and the OpenSSH Include was installed"
        } else {
            ""
        }
    );
    run_agent_once()
}

fn login(arguments: &[String]) -> Result<(), RuntimeError> {
    let [name] = arguments else {
        return Err(RuntimeError::InvalidInput(
            "usage: warpgatesh login <profile>".to_owned(),
        ));
    };
    let store = LocalStore::for_current_user()?;
    let mut catalog = store.load_profiles()?;
    let existing = catalog
        .find(name)
        .cloned()
        .ok_or_else(|| RuntimeError::InvalidInput(format!("unknown profile '{name}'")))?;
    let client = ApiClient::new(&existing.base_url)?;
    let token_page = client.token_page_url()?;
    println!("Opening {token_page}");
    if let Err(error) = open_token_page(token_page.as_str()) {
        eprintln!("warpgatesh: {error}; open the URL above manually");
    }
    let token = prompt_token()?;
    let metadata = client.validate(&token)?;
    catalog.upsert(Profile {
        name: existing.name,
        base_url: existing.base_url,
        username: metadata.username,
        warpgate_version: metadata.version,
        ssh_host: metadata.ssh_host,
        ssh_port: metadata.ssh_port,
    })?;
    SystemKeychain.set(name, &token)?;
    store.save_profiles(&catalog)?;
    println!("Token for profile '{name}' updated.");
    run_agent_once()
}

fn list_profiles() -> Result<(), RuntimeError> {
    let catalog = LocalStore::for_current_user()?.load_profiles()?;
    if catalog.profiles.is_empty() {
        println!("No Warpgate profiles configured.");
        return Ok(());
    }
    for profile in &catalog.profiles {
        let marker = if catalog.is_default(&profile.name) {
            "*"
        } else {
            " "
        };
        println!(
            "{marker} {}\t{}\t{}",
            profile.name, profile.username, profile.base_url
        );
    }
    Ok(())
}

fn set_default_profile(name: &str) -> Result<(), RuntimeError> {
    let store = LocalStore::for_current_user()?;
    let mut catalog = store.load_profiles()?;
    if catalog.find(name).is_none() {
        return Err(RuntimeError::InvalidInput(format!(
            "unknown profile '{name}'"
        )));
    }
    catalog.default_profile = Some(name.to_owned());
    store.save_profiles(&catalog)?;
    println!("Default profile set to '{name}'.");
    run_agent_once()
}

fn list_targets(arguments: &[String]) -> Result<(), RuntimeError> {
    require_no_arguments("ls", arguments)?;
    let snapshot = LocalStore::for_current_user()?.load_snapshot()?;
    let Some(snapshot) = snapshot else {
        println!("No synchronized SSH targets. Run 'warpgatesh sync'.");
        return Ok(());
    };
    for target in snapshot.targets {
        let alias = target
            .short_alias
            .as_deref()
            .unwrap_or(&target.qualified_alias);
        println!("{alias}\t{}\t{}", target.profile, target.name);
    }
    Ok(())
}

fn status(arguments: &[String]) -> Result<(), RuntimeError> {
    require_no_arguments("status", arguments)?;
    let store = LocalStore::for_current_user()?;
    let catalog = store.load_profiles()?;
    println!("Profiles: {}", catalog.profiles.len());
    if let Some(default) = catalog.default_profile {
        println!("Default profile: {default}");
    }
    match store.load_snapshot()? {
        Some(snapshot) => {
            let age = epoch_seconds().saturating_sub(snapshot.synchronized_at_epoch_seconds);
            println!("SSH targets: {}", snapshot.targets.len());
            println!("Last successful sync: {age} seconds ago");
        }
        None => println!("Last successful sync: never"),
    }
    Ok(())
}

fn doctor(arguments: &[String]) -> Result<(), RuntimeError> {
    require_no_arguments("doctor", arguments)?;
    let store = LocalStore::for_current_user()?;
    println!(
        "OpenSSH: {}",
        if Path::new("/usr/bin/ssh").is_file() {
            "ok"
        } else {
            "missing"
        }
    );
    println!("Profiles file: {}", store.paths().profiles.display());
    println!("Managed SSH file: {}", store.paths().ssh_config.display());
    println!(
        "Configured profiles: {}",
        store.load_profiles()?.profiles.len()
    );
    Ok(())
}

fn run_agent_once() -> Result<(), RuntimeError> {
    let current = std::env::current_exe()?;
    let sibling = current
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("warpgatesh-agent");
    let executable = if sibling.is_file() {
        sibling
    } else {
        "warpgatesh-agent".into()
    };
    let status = Command::new(&executable)
        .arg("--once")
        .status()
        .map_err(|error| {
            RuntimeError::Command(format!(
                "could not start {}: {error}; build or install the agent first",
                executable.display()
            ))
        })?;
    if status.success() {
        Ok(())
    } else {
        Err(RuntimeError::Command(format!(
            "synchronization agent failed with {status}"
        )))
    }
}

fn prompt_token() -> Result<String, RuntimeError> {
    let token = rpassword::prompt_password("Paste the personal Warpgate API token: ")?;
    let token = token.trim().to_owned();
    if token.is_empty() {
        Err(RuntimeError::InvalidInput(
            "the API token cannot be empty".to_owned(),
        ))
    } else {
        Ok(token)
    }
}

fn confirm(prompt: &str) -> Result<bool, RuntimeError> {
    print!("{prompt}");
    io::stdout().flush()?;
    let mut answer = String::new();
    io::stdin().read_line(&mut answer)?;
    Ok(matches!(
        answer.trim().to_ascii_lowercase().as_str(),
        "y" | "yes" | "o" | "oui"
    ))
}

fn require_no_arguments(command: &str, arguments: &[String]) -> Result<(), RuntimeError> {
    if arguments.is_empty() {
        Ok(())
    } else {
        Err(RuntimeError::InvalidInput(format!(
            "'{command}' takes no arguments"
        )))
    }
}

fn warn_if_snapshot_is_stale() {
    let Ok(store) = LocalStore::for_current_user() else {
        return;
    };
    let Ok(snapshot) = store.load_snapshot() else {
        return;
    };
    match snapshot {
        None => eprintln!("warpgatesh: warning: no successful synchronization yet"),
        Some(snapshot)
            if epoch_seconds().saturating_sub(snapshot.synchronized_at_epoch_seconds) > 10 * 60 =>
        {
            eprintln!("warpgatesh: warning: the local target snapshot is stale");
        }
        Some(_) => {}
    }
}

fn epoch_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
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
