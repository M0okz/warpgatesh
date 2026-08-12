use std::process::ExitCode;

use warpgatesh_core::schedule::SyncSchedule;
use warpgatesh_runtime::RuntimeError;
use warpgatesh_runtime::configuration::{ConfigurationMutation, LocalConfiguration};
use warpgatesh_runtime::ipc::MUTATION_PREFIX;
use warpgatesh_runtime::keychain::SystemKeychain;
use warpgatesh_runtime::ssh::verify_host_keys;
use warpgatesh_runtime::storage::{
    AGENT_STATUS_SCHEMA_VERSION, AgentErrorKind, AgentStatus, LocalStore,
};
use warpgatesh_runtime::sync::{SyncReport, synchronize_all};

const LOCAL_COMMAND_LIMIT: u64 = 64 * 1024;

#[cfg(unix)]
type SyncWorker = std::thread::JoinHandle<Result<SyncReport, RuntimeError>>;

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
    synchronize_and_record(&store)
}

fn synchronize_and_record(store: &LocalStore) -> Result<SyncReport, RuntimeError> {
    let attempted_at = epoch_seconds();
    let previous_success = store
        .load_agent_status()
        .ok()
        .flatten()
        .and_then(|status| status.last_success_epoch_seconds);
    match verify_and_synchronize(store) {
        Ok(report) => {
            store.save_agent_status(&AgentStatus {
                schema_version: AGENT_STATUS_SCHEMA_VERSION,
                last_attempt_epoch_seconds: attempted_at,
                last_success_epoch_seconds: Some(report.synchronized_at_epoch_seconds),
                last_error_kind: None,
                last_error_message: None,
            })?;
            Ok(report)
        }
        Err(error) => {
            let status = AgentStatus {
                schema_version: AGENT_STATUS_SCHEMA_VERSION,
                last_attempt_epoch_seconds: attempted_at,
                last_success_epoch_seconds: previous_success,
                last_error_kind: Some(error_kind(&error)),
                last_error_message: Some(error.to_string()),
            };
            let _ = store.save_agent_status(&status);
            Err(error)
        }
    }
}

fn verify_and_synchronize(store: &LocalStore) -> Result<SyncReport, RuntimeError> {
    for profile in store.load_profiles()?.profiles {
        verify_host_keys(
            store.paths(),
            &profile.name,
            &profile.ssh_host,
            profile.ssh_port,
        )?;
    }
    synchronize_all(store, &SystemKeychain)
}

fn error_kind(error: &RuntimeError) -> AgentErrorKind {
    match error {
        RuntimeError::Unauthorized | RuntimeError::Keychain(_) => AgentErrorKind::Unauthorized,
        RuntimeError::Http(_) | RuntimeError::Url(_) => AgentErrorKind::ApiUnreachable,
        RuntimeError::Incompatible(_) => AgentErrorKind::Incompatible,
        RuntimeError::Command(message) | RuntimeError::InvalidInput(message)
            if message.to_ascii_lowercase().contains("host key") =>
        {
            AgentErrorKind::HostKey
        }
        _ => AgentErrorKind::Other,
    }
}

fn epoch_seconds() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};

    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
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
    use std::io::{BufRead, BufReader, ErrorKind, Read};
    use std::os::unix::fs::PermissionsExt;
    use std::os::unix::net::{UnixListener, UnixStream};
    use std::thread;
    use std::time::{Duration, Instant};

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

    let mut schedule = schedule_from_preferences(&store)?;
    let mut failures = 0_u32;
    let mut next_sync = Instant::now();
    let mut sync_worker = None;
    let mut resync_requested = false;
    println!(
        "warpgatesh-agent: running (default interval: {} seconds)",
        schedule.interval.as_secs()
    );

    loop {
        if sync_worker.is_none() && (resync_requested || Instant::now() >= next_sync) {
            resync_requested = false;
            sync_worker = Some(spawn_synchronization(&store));
        }

        if sync_worker
            .as_ref()
            .is_some_and(std::thread::JoinHandle::is_finished)
        {
            let result = sync_worker
                .take()
                .expect("finished synchronization worker")
                .join()
                .unwrap_or_else(|_| {
                    Err(RuntimeError::Command(
                        "the synchronization worker stopped unexpectedly".to_owned(),
                    ))
                });
            finish_synchronization(result, &store, &mut failures, &mut schedule, &mut next_sync)?;
        }

        match listener.accept() {
            Ok((mut stream, _)) => {
                let mut command = String::new();
                if let Err(error) = BufReader::new(stream.try_clone()?)
                    .take(LOCAL_COMMAND_LIMIT)
                    .read_line(&mut command)
                {
                    eprintln!("warpgatesh-agent: could not read a local command: {error}");
                    continue;
                }
                let response = handle_local_request(
                    command.trim_end(),
                    &store,
                    &mut schedule,
                    &mut next_sync,
                    &mut sync_worker,
                    &mut resync_requested,
                );
                answer_local_command(&mut stream, &response);
            }
            Err(error) if error.kind() == ErrorKind::WouldBlock => {
                let delay = if sync_worker.is_some() {
                    Duration::from_millis(250)
                } else {
                    next_sync
                        .saturating_duration_since(Instant::now())
                        .min(Duration::from_millis(250))
                };
                thread::sleep(delay);
            }
            Err(error) => return Err(error.into()),
        }
    }
}

#[cfg(unix)]
fn handle_local_request(
    request: &str,
    store: &LocalStore,
    schedule: &mut SyncSchedule,
    next_sync: &mut std::time::Instant,
    sync_worker: &mut Option<SyncWorker>,
    resync_requested: &mut bool,
) -> String {
    if let Some(payload) = request.strip_prefix(MUTATION_PREFIX) {
        return match apply_configuration(store, payload) {
            Ok(()) => match schedule_from_preferences(store) {
                Ok(updated) => {
                    *schedule = updated;
                    *resync_requested = true;
                    "ok configuration saved".to_owned()
                }
                Err(error) => format!("error {}", protocol_text(&error.to_string())),
            },
            Err(error) => format!("error {}", protocol_text(&error.to_string())),
        };
    }

    match request {
        "status" => format_status_response(
            sync_worker.is_some(),
            next_sync
                .saturating_duration_since(std::time::Instant::now())
                .as_secs(),
        ),
        "sync" => {
            if sync_worker.is_none() {
                *sync_worker = Some(spawn_synchronization(store));
                "ok synchronization started".to_owned()
            } else {
                "ok synchronization already running".to_owned()
            }
        }
        "reload" => match schedule_from_preferences(store) {
            Ok(updated) => {
                *schedule = updated;
                *next_sync = std::time::Instant::now() + schedule.interval;
                format!("ok interval {}", schedule.interval.as_secs())
            }
            Err(error) => format!("error {}", protocol_text(&error.to_string())),
        },
        _ => "error unknown agent command".to_owned(),
    }
}

fn format_status_response(synchronizing: bool, next_sync_seconds: u64) -> String {
    let state = if synchronizing {
        "synchronizing"
    } else {
        "idle"
    };
    format!("ok running state={state} next_sync_seconds={next_sync_seconds}")
}

#[cfg(unix)]
fn apply_configuration(store: &LocalStore, payload: &str) -> Result<(), RuntimeError> {
    let mutation = ConfigurationMutation::from_json(payload)?;
    LocalConfiguration::new(store, &SystemKeychain).apply(mutation)
}

#[cfg(unix)]
fn answer_local_command(stream: &mut std::os::unix::net::UnixStream, response: &str) {
    use std::io::Write;

    if let Err(error) = stream
        .write_all(response.as_bytes())
        .and_then(|()| stream.write_all(b"\n"))
    {
        eprintln!("warpgatesh-agent: could not answer a local command: {error}");
    }
}

#[cfg(unix)]
fn spawn_synchronization(store: &LocalStore) -> SyncWorker {
    let store = store.clone();
    std::thread::spawn(move || synchronize_and_record(&store))
}

#[cfg(unix)]
fn finish_synchronization(
    result: Result<SyncReport, RuntimeError>,
    store: &LocalStore,
    failures: &mut u32,
    schedule: &mut SyncSchedule,
    next_sync: &mut std::time::Instant,
) -> Result<(), RuntimeError> {
    match result {
        Ok(report) => {
            *failures = 0;
            *schedule = schedule_from_preferences(store)?;
            println!("{}", format_report(&report));
            *next_sync = std::time::Instant::now() + schedule.periodic_delay(epoch_seconds());
        }
        Err(error) => {
            *failures = failures.saturating_add(1);
            let delay = schedule.retry_delay(*failures);
            eprintln!(
                "warpgatesh-agent: synchronization failed: {error}; retrying in {} seconds",
                delay.as_secs()
            );
            *next_sync = std::time::Instant::now() + delay;
        }
    }
    Ok(())
}

fn schedule_from_preferences(store: &LocalStore) -> Result<SyncSchedule, RuntimeError> {
    use std::time::Duration;

    let preferences = store.load_preferences()?;
    Ok(SyncSchedule::with_interval(Duration::from_secs(
        preferences.sync_interval_seconds,
    )))
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

    #[test]
    fn reports_the_live_agent_schedule() {
        assert_eq!(
            format_status_response(false, 83),
            "ok running state=idle next_sync_seconds=83"
        );
        assert_eq!(
            format_status_response(true, 0),
            "ok running state=synchronizing next_sync_seconds=0"
        );
    }
}
