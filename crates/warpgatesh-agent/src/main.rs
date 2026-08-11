use std::process::ExitCode;

use warpgatesh_core::schedule::SyncSchedule;
use warpgatesh_runtime::RuntimeError;
use warpgatesh_runtime::keychain::SystemKeychain;
use warpgatesh_runtime::storage::LocalStore;
use warpgatesh_runtime::sync::{SyncReport, synchronize_all};

fn main() -> ExitCode {
    match std::env::args().nth(1).as_deref() {
        Some("--help" | "-h") => {
            println!("Usage: warpgatesh-agent [--once]");
            ExitCode::SUCCESS
        }
        Some("--once") => match synchronize_once() {
            Ok(report) => {
                println!("{}", format_report(&report));
                ExitCode::SUCCESS
            }
            Err(error) => {
                eprintln!("warpgatesh-agent: {error}");
                ExitCode::FAILURE
            }
        },
        Some(argument) => {
            eprintln!("warpgatesh-agent: unknown argument '{argument}'");
            ExitCode::from(2)
        }
        None => match run_forever() {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("warpgatesh-agent: {error}");
                ExitCode::FAILURE
            }
        },
    }
}

fn synchronize_once() -> Result<SyncReport, RuntimeError> {
    let store = LocalStore::for_current_user()?;
    synchronize_all(&store, &SystemKeychain)
}

fn format_report(report: &SyncReport) -> String {
    format!(
        "Synchronized {} SSH target(s) from {} profile(s): +{}, -{}",
        report.target_count, report.profile_count, report.added, report.removed
    )
}

#[cfg(unix)]
fn run_forever() -> Result<(), RuntimeError> {
    use std::fs;
    use std::io::{BufRead, BufReader, ErrorKind, Read, Write};
    use std::os::unix::fs::PermissionsExt;
    use std::os::unix::net::{UnixListener, UnixStream};
    use std::thread;
    use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

    let store = LocalStore::for_current_user()?;
    let socket = &store.paths().agent_socket;
    let parent = socket.parent().ok_or_else(|| {
        RuntimeError::InvalidInput("the agent socket has no parent directory".to_owned())
    })?;
    fs::create_dir_all(parent)?;
    fs::set_permissions(parent, fs::Permissions::from_mode(0o700))?;

    if socket.exists() {
        if UnixStream::connect(socket).is_ok() {
            return Err(RuntimeError::Command(
                "another synchronization agent is already running".to_owned(),
            ));
        }
        fs::remove_file(socket)?;
    }

    let listener = UnixListener::bind(socket)?;
    fs::set_permissions(socket, fs::Permissions::from_mode(0o600))?;
    listener.set_nonblocking(true)?;

    let schedule = SyncSchedule::default();
    let mut failures = 0_u32;
    let mut next_sync = Instant::now();
    println!(
        "warpgatesh-agent: running (default interval: {} seconds)",
        schedule.interval.as_secs()
    );

    loop {
        if Instant::now() >= next_sync {
            match synchronize_all(&store, &SystemKeychain) {
                Ok(report) => {
                    failures = 0;
                    println!("{}", format_report(&report));
                    let seed = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs();
                    next_sync = Instant::now() + schedule.periodic_delay(seed);
                }
                Err(error) => {
                    failures = failures.saturating_add(1);
                    let delay = schedule.retry_delay(failures);
                    eprintln!(
                        "warpgatesh-agent: synchronization failed: {error}; retrying in {} seconds",
                        delay.as_secs()
                    );
                    next_sync = Instant::now() + delay;
                }
            }
        }

        match listener.accept() {
            Ok((mut stream, _)) => {
                let mut command = String::new();
                BufReader::new(stream.try_clone()?)
                    .take(1024)
                    .read_line(&mut command)?;
                let response = match command.trim_end() {
                    "status" => "ok running".to_owned(),
                    "sync" => match synchronize_all(&store, &SystemKeychain) {
                        Ok(report) => {
                            failures = 0;
                            let seed = SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_secs();
                            next_sync = Instant::now() + schedule.periodic_delay(seed);
                            format!("ok {}", format_report(&report))
                        }
                        Err(error) => format!("error {}", protocol_text(&error.to_string())),
                    },
                    _ => "error unknown agent command".to_owned(),
                };
                stream.write_all(response.as_bytes())?;
                stream.write_all(b"\n")?;
            }
            Err(error) if error.kind() == ErrorKind::WouldBlock => {
                let remaining = next_sync.saturating_duration_since(Instant::now());
                thread::sleep(remaining.min(Duration::from_millis(250)));
            }
            Err(error) => return Err(error.into()),
        }
    }
}

#[cfg(not(unix))]
fn run_forever() -> Result<(), RuntimeError> {
    Err(RuntimeError::Command(
        "the persistent synchronization agent requires a Unix platform".to_owned(),
    ))
}

#[cfg(unix)]
fn protocol_text(message: &str) -> String {
    message.replace(['\n', '\r'], " ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_a_human_readable_sync_report() {
        let report = SyncReport {
            profile_count: 2,
            target_count: 12,
            added: 3,
            removed: 1,
            synchronized_at_epoch_seconds: 0,
        };
        assert_eq!(
            format_report(&report),
            "Synchronized 12 SSH target(s) from 2 profile(s): +3, -1"
        );
    }
}
