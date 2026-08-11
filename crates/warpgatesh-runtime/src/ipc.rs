use std::io::{Read, Write};
use std::path::Path;
use std::time::{Duration, Instant};

use crate::RuntimeError;
use crate::configuration::ConfigurationMutation;

const RESPONSE_LIMIT: u64 = 16 * 1024;
pub const MUTATION_PREFIX: &str = "mutate ";

/// Send one command to the per-user synchronization agent.
///
/// # Errors
///
/// Returns [`RuntimeError`] when the socket is unavailable, the response is
/// invalid, or the agent reports an error.
#[cfg(unix)]
pub fn request(path: &Path, command: &str) -> Result<String, RuntimeError> {
    request_with_read_timeout(path, command, Duration::from_secs(120))
}

/// Send one command with a bounded response wait.
///
/// # Errors
///
/// Returns [`RuntimeError`] when the socket is unavailable, the response is
/// invalid, or the agent does not answer before `read_timeout`.
#[cfg(unix)]
pub fn request_with_read_timeout(
    path: &Path,
    command: &str,
    read_timeout: Duration,
) -> Result<String, RuntimeError> {
    use std::net::Shutdown;
    use std::os::unix::net::UnixStream;

    if command.is_empty() || command.contains('\n') || command.contains('\r') {
        return Err(RuntimeError::InvalidInput(
            "invalid agent command".to_owned(),
        ));
    }

    let mut stream = UnixStream::connect(path)?;
    stream.set_read_timeout(Some(read_timeout))?;
    stream.set_write_timeout(Some(Duration::from_secs(5)))?;
    stream.write_all(command.as_bytes())?;
    stream.write_all(b"\n")?;
    stream.shutdown(Shutdown::Write)?;

    let mut response = String::new();
    stream.take(RESPONSE_LIMIT).read_to_string(&mut response)?;
    let response = response.trim_end();
    if let Some(message) = response.strip_prefix("ok ") {
        Ok(message.to_owned())
    } else if let Some(message) = response.strip_prefix("error ") {
        Err(RuntimeError::Command(message.to_owned()))
    } else {
        Err(RuntimeError::Incompatible(
            "the synchronization agent returned an invalid response".to_owned(),
        ))
    }
}

#[cfg(not(unix))]
pub fn request_with_read_timeout(
    _path: &Path,
    _command: &str,
    _read_timeout: Duration,
) -> Result<String, RuntimeError> {
    Err(RuntimeError::Command(
        "local agent IPC is unsupported on this platform".to_owned(),
    ))
}

#[cfg(not(unix))]
pub fn request(_path: &Path, _command: &str) -> Result<String, RuntimeError> {
    Err(RuntimeError::Command(
        "local agent IPC is unsupported on this platform".to_owned(),
    ))
}

/// Send a command while allowing a newly launched agent time to create its
/// socket.
///
/// # Errors
///
/// Returns [`RuntimeError`] immediately for protocol and agent errors, or after
/// `timeout` when the socket remains absent or refuses connections.
pub fn request_with_retry(
    path: &Path,
    command: &str,
    timeout: Duration,
) -> Result<String, RuntimeError> {
    use std::io::ErrorKind;
    use std::thread;

    let deadline = Instant::now() + timeout;
    loop {
        match request(path, command) {
            Err(RuntimeError::Io(error))
                if matches!(
                    error.kind(),
                    ErrorKind::NotFound | ErrorKind::ConnectionRefused
                ) && Instant::now() < deadline =>
            {
                let remaining = deadline.saturating_duration_since(Instant::now());
                thread::sleep(remaining.min(Duration::from_millis(100)));
            }
            result => return result,
        }
    }
}

/// Send one typed configuration mutation to the background agent.
///
/// # Errors
///
/// Returns [`RuntimeError`] when serialization fails or the agent rejects the
/// mutation.
pub fn request_mutation(
    path: &Path,
    mutation: &ConfigurationMutation,
    timeout: Duration,
) -> Result<String, RuntimeError> {
    let payload = serde_json::to_string(mutation)?;
    request_with_retry(path, &format!("{MUTATION_PREFIX}{payload}"), timeout)
}

#[cfg(all(test, unix))]
mod tests {
    use std::fs;
    use std::os::unix::net::UnixListener;
    use std::thread;

    use tempfile::TempDir;
    use warpgatesh_core::profiles::Profile;

    use super::*;

    #[test]
    fn exchanges_a_bounded_line_with_the_agent() {
        let directory = TempDir::new().expect("temporary directory");
        let socket = directory.path().join("agent.sock");
        let listener = UnixListener::bind(&socket).expect("bind socket");
        let worker = thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("accept request");
            let mut command = String::new();
            stream.read_to_string(&mut command).expect("read command");
            assert_eq!(command, "sync\n");
            stream.write_all(b"ok synchronized\n").expect("response");
        });

        assert_eq!(
            request(&socket, "sync").expect("agent response"),
            "synchronized"
        );
        worker.join().expect("server thread");
        fs::remove_file(socket).expect("remove socket");
    }

    #[test]
    fn propagates_an_agent_error() {
        let directory = TempDir::new().expect("temporary directory");
        let socket = directory.path().join("agent.sock");
        let listener = UnixListener::bind(&socket).expect("bind socket");
        let worker = thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("accept request");
            let mut command = String::new();
            stream.read_to_string(&mut command).expect("read command");
            stream
                .write_all(b"error token expired\n")
                .expect("response");
        });

        let error = request(&socket, "sync").expect_err("agent error");
        assert!(error.to_string().contains("token expired"));
        worker.join().expect("server thread");
    }

    #[test]
    fn waits_for_a_socket_created_during_agent_startup() {
        let directory = TempDir::new().expect("temporary directory");
        let socket = directory.path().join("agent.sock");
        let server_socket = socket.clone();
        let worker = thread::spawn(move || {
            thread::sleep(Duration::from_millis(75));
            let listener = UnixListener::bind(server_socket).expect("bind delayed socket");
            let (mut stream, _) = listener.accept().expect("accept request");
            let mut command = String::new();
            stream.read_to_string(&mut command).expect("read command");
            stream.write_all(b"ok synchronized\n").expect("response");
        });

        assert_eq!(
            request_with_retry(&socket, "sync", Duration::from_secs(1))
                .expect("delayed agent response"),
            "synchronized"
        );
        worker.join().expect("server thread");
    }

    #[test]
    fn sends_a_typed_configuration_mutation() {
        let directory = TempDir::new().expect("temporary directory");
        let socket = directory.path().join("agent.sock");
        let listener = UnixListener::bind(&socket).expect("bind socket");
        let worker = thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("accept request");
            let mut command = String::new();
            stream.read_to_string(&mut command).expect("read command");
            let payload = command
                .trim_end()
                .strip_prefix(MUTATION_PREFIX)
                .expect("mutation prefix");
            let mutation = ConfigurationMutation::from_json(payload).expect("typed mutation");
            assert!(matches!(
                mutation,
                ConfigurationMutation::SaveProfile { profile, token, .. }
                    if profile.name == "lab" && token == "secret"
            ));
            stream
                .write_all(b"ok configuration saved\n")
                .expect("response");
        });
        let mutation = ConfigurationMutation::SaveProfile {
            profile: Profile {
                name: "lab".to_owned(),
                base_url: "https://warpgate.example/".to_owned(),
                username: "gregory".to_owned(),
                warpgate_version: None,
                ssh_host: "ssh.example".to_owned(),
                ssh_port: 2222,
            },
            token: "secret".to_owned(),
            known_hosts: "ssh.example ssh-ed25519 AAAA\n".to_owned(),
        };

        assert_eq!(
            request_mutation(&socket, &mutation, Duration::from_secs(1))
                .expect("mutation response"),
            "configuration saved"
        );
        worker.join().expect("server thread");
    }
}
