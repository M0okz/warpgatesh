//! Synthetic responses derived from upstream `OpenAPI` schemas; no live credentials.

use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::thread;

use tempfile::TempDir;
use warpgatesh_core::paths::WarpgatePaths;
use warpgatesh_core::profiles::{Profile, ProfileCatalog, SshAuthentication};
use warpgatesh_runtime::RuntimeError;
use warpgatesh_runtime::api::ApiClient;
use warpgatesh_runtime::keychain::TokenStore;
use warpgatesh_runtime::storage::{LocalStore, atomic_write};
use warpgatesh_runtime::sync::synchronize_all;

const CONTRACTS: [(&str, &str, &str); 3] = [
    (
        "0.27.5",
        include_str!("fixtures/warpgate-0.27.5/info.json"),
        include_str!("fixtures/warpgate-0.27.5/targets.json"),
    ),
    (
        "0.28.6",
        include_str!("fixtures/warpgate-0.28.6/info.json"),
        include_str!("fixtures/warpgate-0.28.6/targets.json"),
    ),
    (
        "0.29.1",
        include_str!("fixtures/warpgate-0.29.1/info.json"),
        include_str!("fixtures/warpgate-0.29.1/targets.json"),
    ),
];

const SSH_TARGET_ID: &str = "00000000-0000-4000-8000-000000000004";

struct Server {
    url: String,
    requests: Arc<Mutex<Vec<String>>>,
    handle: thread::JoinHandle<()>,
}

impl Server {
    fn start(responses: Vec<(u16, String)>) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("mock listener");
        let url = format!("http://{}/", listener.local_addr().expect("mock address"));
        let requests = Arc::new(Mutex::new(Vec::new()));
        let captured = Arc::clone(&requests);
        let handle = thread::spawn(move || {
            for (status, body) in responses {
                let (mut stream, _) = listener.accept().expect("accept request");
                let mut request = Vec::new();
                while !request.ends_with(b"\r\n\r\n") {
                    let mut byte = [0_u8];
                    stream.read_exact(&mut byte).expect("read request header");
                    request.push(byte[0]);
                }
                captured
                    .lock()
                    .expect("requests")
                    .push(String::from_utf8(request).expect("HTTP request"));
                write!(
                    stream,
                    "HTTP/1.1 {status} Mock\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                )
                .expect("write response");
            }
        });
        Self {
            url,
            requests,
            handle,
        }
    }

    fn finish(self) -> Vec<String> {
        self.handle.join().expect("mock server");
        Arc::try_unwrap(self.requests)
            .expect("sole request owner")
            .into_inner()
            .expect("requests")
    }
}

struct MemoryToken;

impl TokenStore for MemoryToken {
    fn set(&self, _profile: &str, _token: &str) -> Result<(), RuntimeError> {
        Ok(())
    }

    fn get(&self, _profile: &str) -> Result<String, RuntimeError> {
        Ok("synthetic-token".to_owned())
    }

    fn delete(&self, _profile: &str) -> Result<(), RuntimeError> {
        Ok(())
    }
}

fn configure_store(home: &TempDir, url: String) -> LocalStore {
    let store = LocalStore::new(WarpgatePaths::for_home(home.path()));
    let mut catalog = ProfileCatalog::default();
    catalog
        .upsert(Profile {
            name: "lab".to_owned(),
            base_url: url,
            username: "alice".to_owned(),
            warpgate_version: Some("0.27.5".to_owned()),
            ssh_host: "ssh.example.test".to_owned(),
            ssh_port: 2222,
            ssh_authentication: SshAuthentication::InBrowser,
        })
        .expect("profile");
    store.save_profiles(&catalog).expect("save profiles");
    atomic_write(
        &store.paths().known_hosts_directory.join("lab"),
        b"synthetic-pinned-key\n",
    )
    .expect("pinned key");
    store
}

#[test]
fn reads_user_api_contracts_before_and_after_029() {
    for (version, info, targets) in CONTRACTS {
        let server = Server::start(vec![(200, info.to_owned()), (200, targets.to_owned())]);
        let client = ApiClient::new(&server.url).expect("client");
        let metadata = client.validate("synthetic-token").expect("metadata");
        let targets = client.ssh_targets("synthetic-token").expect("targets");
        let requests = server.finish();

        assert_eq!(metadata.version.as_deref(), Some(version));
        assert_eq!(metadata.username, "alice");
        assert_eq!(metadata.ssh_host, "ssh.example.test");
        assert_eq!(metadata.ssh_port, 2222);
        assert_eq!(targets.len(), 1, "only SSH targets for {version}");
        assert_eq!(targets[0].id, SSH_TARGET_ID);
        assert_eq!(targets[0].name, "db");
        for (request, path) in requests.iter().zip(["info", "targets"]) {
            assert!(request.starts_with(&format!("GET /@warpgate/api/{path} HTTP/1.1\r\n")));
            assert!(
                request
                    .to_ascii_lowercase()
                    .contains("\r\nx-warpgate-token: synthetic-token\r\n")
            );
        }
    }
}

#[test]
fn synchronizes_each_contract_without_changing_ssh_browser_authentication() {
    for (version, info, targets) in CONTRACTS {
        let server = Server::start(vec![(200, info.to_owned()), (200, targets.to_owned())]);
        let home = TempDir::new().expect("temporary home");
        let store = configure_store(&home, server.url.clone());
        let report = synchronize_all(&store, &MemoryToken).expect("synchronize");
        server.finish();

        assert_eq!(report.target_count, 1);
        let profile = store.load_profiles().expect("profiles").profiles.remove(0);
        assert_eq!(profile.warpgate_version.as_deref(), Some(version));
        assert_eq!(profile.ssh_authentication, SshAuthentication::InBrowser);
        let config = fs::read_to_string(&store.paths().ssh_config).expect("SSH config");
        assert!(config.contains("Host db db.lab"));
        assert!(config.contains("User \"alice:db\""));
        assert!(config.contains("HostName \"ssh.example.test\""));
        assert!(config.contains("PreferredAuthentications keyboard-interactive"));
        assert!(!config.contains("synthetic-token"));
        let snapshot = store.load_snapshot().expect("snapshot").expect("exists");
        assert_eq!(snapshot.targets[0].target_id, SSH_TARGET_ID);
    }
}

#[test]
fn preserves_saved_state_when_the_upgraded_api_fails() {
    let (_, info, targets) = CONTRACTS[2];
    for (status, body) in [
        (200, "{}"),
        (200, r#"[{"id":"incomplete","kind":"Ssh"}]"#),
        (401, r#""Unauthenticated""#),
        (403, r#""Forbidden""#),
        (500, r#""Internal error (reference: synthetic-reference)""#),
    ] {
        // A successful baseline followed by an incompatible or rejected response.
        let server = Server::start(vec![
            (200, info.to_owned()),
            (200, targets.to_owned()),
            (200, info.to_owned()),
            (status, body.to_owned()),
        ]);
        let home = TempDir::new().expect("temporary home");
        let store = configure_store(&home, server.url.clone());
        synchronize_all(&store, &MemoryToken).expect("baseline");
        let paths = [
            &store.paths().ssh_config,
            &store.paths().snapshot,
            &store.paths().profiles,
        ];
        let before: Vec<_> = paths.iter().map(|p| fs::read(p).expect("state")).collect();

        let error = synchronize_all(&store, &MemoryToken).expect_err("reject failed response");
        server.finish();
        if matches!(status, 401 | 403) {
            assert!(matches!(error, RuntimeError::Unauthorized));
        }
        let after: Vec<_> = paths.iter().map(|p| fs::read(p).expect("state")).collect();
        assert_eq!(before, after, "preserve all saved state for HTTP {status}");
    }
}
