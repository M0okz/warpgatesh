use std::time::Duration;

use reqwest::StatusCode;
use reqwest::blocking::{Client, Response};
use serde::Deserialize;
use url::Url;
use warpgatesh_core::aliases::Target;

use crate::RuntimeError;

const TOKEN_HEADER: &str = "x-warpgate-token";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InstanceMetadata {
    pub username: String,
    pub version: Option<String>,
    pub ssh_host: String,
    pub ssh_port: u16,
}

#[derive(Debug, Deserialize)]
struct InfoResponse {
    version: Option<String>,
    username: Option<String>,
    external_host: Option<String>,
    external_hosts: Option<ExternalHosts>,
    ports: Ports,
}

#[derive(Debug, Deserialize)]
struct ExternalHosts {
    ssh: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Ports {
    ssh: Option<u16>,
}

#[derive(Debug, Deserialize)]
struct ApiTarget {
    id: String,
    name: String,
    kind: String,
}

#[derive(Clone, Debug)]
pub struct ApiClient {
    client: Client,
    base_url: Url,
}

impl ApiClient {
    /// Create a client for an instance root URL.
    ///
    /// # Errors
    ///
    /// Returns [`RuntimeError`] when the URL or HTTP client is invalid.
    pub fn new(base_url: &str) -> Result<Self, RuntimeError> {
        let base_url = normalize_base_url(base_url)?;
        let client = Client::builder()
            .timeout(Duration::from_secs(15))
            .user_agent(concat!("WarpgateSH/", env!("CARGO_PKG_VERSION")))
            .build()?;
        Ok(Self { client, base_url })
    }

    #[must_use]
    pub fn base_url(&self) -> &Url {
        &self.base_url
    }

    /// Return the browser page where a user creates personal API tokens.
    ///
    /// # Errors
    ///
    /// Returns [`RuntimeError`] if the route cannot be joined to the base URL.
    pub fn token_page_url(&self) -> Result<Url, RuntimeError> {
        Ok(self.base_url.join("@warpgate/#/profile/api-tokens")?)
    }

    /// Validate a token and discover the authenticated SSH endpoint.
    ///
    /// # Errors
    ///
    /// Returns [`RuntimeError`] for authentication, transport, schema, or
    /// missing SSH capability errors.
    pub fn validate(&self, token: &str) -> Result<InstanceMetadata, RuntimeError> {
        let url = self.base_url.join("@warpgate/api/info")?;
        let info: InfoResponse =
            checked(self.client.get(url).header(TOKEN_HEADER, token).send()?)?.json()?;

        let username = info.username.ok_or_else(|| {
            RuntimeError::Incompatible("authenticated /info response has no username".to_owned())
        })?;
        let ssh_host = info
            .external_hosts
            .and_then(|hosts| hosts.ssh)
            .or(info.external_host)
            .ok_or_else(|| RuntimeError::Incompatible("instance exposes no SSH host".to_owned()))?;
        let ssh_port = info.ports.ssh.ok_or_else(|| {
            RuntimeError::Incompatible(
                "SSH protocol is disabled or has no external port".to_owned(),
            )
        })?;

        Ok(InstanceMetadata {
            username,
            version: info.version,
            ssh_host,
            ssh_port,
        })
    }

    /// Retrieve only the SSH targets accessible with the supplied token.
    ///
    /// # Errors
    ///
    /// Returns [`RuntimeError`] for authentication, transport, or schema errors.
    pub fn ssh_targets(&self, token: &str) -> Result<Vec<Target>, RuntimeError> {
        let url = self.base_url.join("@warpgate/api/targets")?;
        let targets: Vec<ApiTarget> =
            checked(self.client.get(url).header(TOKEN_HEADER, token).send()?)?.json()?;

        Ok(targets
            .into_iter()
            .filter(|target| target.kind == "Ssh")
            .map(|target| Target {
                id: target.id,
                name: target.name,
            })
            .collect())
    }
}

fn checked(response: Response) -> Result<Response, RuntimeError> {
    if matches!(
        response.status(),
        StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN
    ) {
        return Err(RuntimeError::Unauthorized);
    }
    Ok(response.error_for_status()?)
}

fn normalize_base_url(value: &str) -> Result<Url, RuntimeError> {
    let mut url = Url::parse(value)?;
    if !matches!(url.scheme(), "http" | "https") || url.host_str().is_none() {
        return Err(RuntimeError::InvalidInput(
            "the Warpgate URL must be an absolute http:// or https:// URL".to_owned(),
        ));
    }

    url.set_query(None);
    url.set_fragment(None);
    if let Some((prefix, _)) = url.path().split_once("/@warpgate") {
        url.set_path(&format!("{prefix}/"));
    } else if !url.path().ends_with('/') {
        let path = format!("{}/", url.path());
        url.set_path(&path);
    }
    Ok(url)
}

#[cfg(test)]
mod tests {
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::sync::{Arc, Mutex};
    use std::thread;

    use super::*;

    fn mock_server(
        responses: Vec<&'static str>,
    ) -> (String, Arc<Mutex<Vec<String>>>, thread::JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind mock server");
        let address = listener.local_addr().expect("mock address");
        let requests = Arc::new(Mutex::new(Vec::new()));
        let captured = Arc::clone(&requests);
        let handle = thread::spawn(move || {
            for body in responses {
                let (mut stream, _) = listener.accept().expect("accept request");
                let mut buffer = [0_u8; 8192];
                let length = stream.read(&mut buffer).expect("read request");
                captured
                    .lock()
                    .expect("capture request")
                    .push(String::from_utf8_lossy(&buffer[..length]).into_owned());
                write!(
                    stream,
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                )
                .expect("write response");
            }
        });
        (format!("http://{address}"), requests, handle)
    }

    #[test]
    fn normalizes_a_copied_warpgate_page_url() {
        let client = ApiClient::new("https://example.test/@warpgate/#/profile/api-tokens")
            .expect("valid client");
        assert_eq!(client.base_url().as_str(), "https://example.test/");
        assert_eq!(
            client.token_page_url().expect("token URL").as_str(),
            "https://example.test/@warpgate/#/profile/api-tokens"
        );
    }

    #[test]
    fn validates_authenticated_instance_metadata() {
        let body = r#"{
            "version":"0.27.0",
            "username":"gregory",
            "external_host":"fallback.example",
            "external_hosts":{"ssh":"ssh.example"},
            "ports":{"ssh":2222}
        }"#;
        let (url, requests, handle) = mock_server(vec![body]);
        let metadata = ApiClient::new(&url)
            .expect("client")
            .validate("secret-token")
            .expect("valid metadata");
        handle.join().expect("mock server");

        assert_eq!(metadata.username, "gregory");
        assert_eq!(metadata.ssh_host, "ssh.example");
        assert_eq!(metadata.ssh_port, 2222);
        assert!(
            requests.lock().expect("requests")[0]
                .to_ascii_lowercase()
                .contains("x-warpgate-token: secret-token")
        );
    }

    #[test]
    fn filters_non_ssh_targets() {
        let body = r#"[
            {"id":"1","name":"server","kind":"Ssh"},
            {"id":"2","name":"website","kind":"Http"}
        ]"#;
        let (url, _, handle) = mock_server(vec![body]);
        let targets = ApiClient::new(&url)
            .expect("client")
            .ssh_targets("token")
            .expect("targets");
        handle.join().expect("mock server");

        assert_eq!(
            targets,
            vec![Target {
                id: "1".to_owned(),
                name: "server".to_owned()
            }]
        );
    }
}
