use std::io::{Read, Write};
use std::path::Path;
use std::time::Duration;

use crate::RuntimeError;

const RESPONSE_LIMIT: u64 = 16 * 1024;

/// Send one command to the per-user synchronization agent.
///
/// # Errors
///
/// Returns [`RuntimeError`] when the socket is unavailable, the response is
/// invalid, or the agent reports an error.
#[cfg(unix)]
pub fn request(path: &Path, command: &str) -> Result<String, RuntimeError> {
    use std::net::Shutdown;
    use std::os::unix::net::UnixStream;

    if command.is_empty() || command.contains('\n') || command.contains('\r') {
        return Err(RuntimeError::InvalidInput(
            "invalid agent command".to_owned(),
        ));
    }

    let mut stream = UnixStream::connect(path)?;
    stream.set_read_timeout(Some(Duration::from_secs(120)))?;
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
pub fn request(_path: &Path, _command: &str) -> Result<String, RuntimeError> {
    Err(RuntimeError::Command(
        "local agent IPC is unsupported on this platform".to_owned(),
    ))
}

#[cfg(all(test, unix))]
mod tests {
    use std::fs;
    use std::os::unix::net::UnixListener;
    use std::thread;

    use tempfile::TempDir;

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
}
