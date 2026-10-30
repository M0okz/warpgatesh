//! The authenticated, origin-bound API used to issue and rotate personal tokens.
use crate::RuntimeError;
use crate::api::{ApiClient, InstanceMetadata, checked};
use chrono::{DateTime, Utc};
use reqwest::blocking::RequestBuilder;
use reqwest::header::{COOKIE, HeaderValue};
use reqwest::{Method, StatusCode};
use serde::{Deserialize, Serialize};
use std::io::Read;

// Deliberately no Debug implementation: a session cookie and an API token are secrets.
#[derive(Clone, Deserialize, Serialize)]
pub enum ApiAuthorization {
    Token(String),
    Session(String),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TokenMetadata {
    pub id: String,
    pub label: String,
    pub created: String,
    pub expiry: String,
}
impl TokenMetadata {
    /// Parse the server's authoritative expiry timestamp.
    /// # Errors
    /// Rejects an invalid date instead of inventing an expiry.
    pub fn expiry_epoch(&self) -> Result<u64, RuntimeError> {
        u64::try_from(
            DateTime::parse_from_rfc3339(&self.expiry)
                .map_err(|_| RuntimeError::Incompatible("invalid token expiry".to_owned()))?
                .timestamp(),
        )
        .map_err(|_| RuntimeError::Incompatible("invalid token expiry".to_owned()))
    }
}
#[derive(Clone, Deserialize, Serialize)]
pub struct IssuedToken {
    pub token: TokenMetadata,
    pub secret: String,
}

pub trait TokenApi {
    /// # Errors
    /// Returns authentication, transport or incompatible-response errors.
    fn identity(&self, authorization: &ApiAuthorization) -> Result<InstanceMetadata, RuntimeError>;
    /// # Errors
    /// Returns authentication, transport or incompatible-response errors.
    fn maximum_duration(
        &self,
        authorization: &ApiAuthorization,
    ) -> Result<Option<u64>, RuntimeError>;
    /// # Errors
    /// Returns authentication, transport or incompatible-response errors.
    fn issue(
        &self,
        authorization: &ApiAuthorization,
        label: &str,
        expiry_epoch: u64,
    ) -> Result<IssuedToken, RuntimeError>;
    /// # Errors
    /// Returns authentication, transport or incompatible-response errors.
    fn list(&self, authorization: &ApiAuthorization) -> Result<Vec<TokenMetadata>, RuntimeError>;
    /// # Errors
    /// Returns authentication, transport or incompatible-response errors.
    fn revoke(&self, authorization: &ApiAuthorization, id: &str) -> Result<(), RuntimeError>;
}

impl ApiClient {
    pub(crate) fn authorized_request(
        &self,
        method: Method,
        path: &str,
        authorization: &ApiAuthorization,
    ) -> Result<RequestBuilder, RuntimeError> {
        if matches!(authorization, ApiAuthorization::Session(_)) {
            self.require_managed_transport()?;
        }
        let (name, value) = match authorization {
            ApiAuthorization::Token(value) => ("x-warpgate-token", value),
            ApiAuthorization::Session(value) => (COOKIE.as_str(), value),
        };
        let mut header = HeaderValue::from_str(value)
            .map_err(|_| RuntimeError::InvalidInput("invalid authentication value".to_owned()))?;
        header.set_sensitive(true);
        Ok(self
            .client
            .request(method, self.base_url.join(path)?)
            .header(name, header))
    }
    fn require_managed_transport(&self) -> Result<(), RuntimeError> {
        if self.base_url.scheme() != "https"
            && !matches!(
                self.base_url.host_str(),
                Some("localhost" | "127.0.0.1" | "::1")
            )
        {
            return Err(RuntimeError::InvalidInput(
                "automatic token management requires HTTPS".to_owned(),
            ));
        }
        Ok(())
    }
    fn token_json<T: serde::de::DeserializeOwned>(
        &self,
        request: RequestBuilder,
    ) -> Result<T, RuntimeError> {
        self.require_managed_transport()?;
        let response = checked(request.send()?)?;
        let mut bytes = Vec::new();
        response.take(65_537).read_to_end(&mut bytes)?;
        if bytes.len() > 65_536 {
            return Err(RuntimeError::Incompatible(
                "token response too large".to_owned(),
            ));
        }
        serde_json::from_slice(&bytes)
            .map_err(|_| RuntimeError::Incompatible("invalid token API response".to_owned()))
    }
}
impl TokenApi for ApiClient {
    fn identity(&self, authorization: &ApiAuthorization) -> Result<InstanceMetadata, RuntimeError> {
        self.require_managed_transport()?;
        self.validate_authorization(authorization)
    }
    fn maximum_duration(
        &self,
        authorization: &ApiAuthorization,
    ) -> Result<Option<u64>, RuntimeError> {
        #[derive(Deserialize)]
        struct Policy {
            max_api_token_duration_seconds: Option<u64>,
        }
        let policy: Policy = self.token_json(self.authorized_request(
            Method::GET,
            "@warpgate/api/info",
            authorization,
        )?)?;
        Ok(policy.max_api_token_duration_seconds)
    }
    fn issue(
        &self,
        authorization: &ApiAuthorization,
        label: &str,
        expiry_epoch: u64,
    ) -> Result<IssuedToken, RuntimeError> {
        let seconds = i64::try_from(expiry_epoch)
            .map_err(|_| RuntimeError::InvalidInput("invalid token expiry".to_owned()))?;
        let expiry = DateTime::<Utc>::from_timestamp(seconds, 0)
            .ok_or_else(|| RuntimeError::InvalidInput("invalid token expiry".to_owned()))?
            .to_rfc3339();
        let token: IssuedToken = self.token_json(
            self.authorized_request(
                Method::POST,
                "@warpgate/api/profile/api-tokens",
                authorization,
            )?
            .json(&serde_json::json!({"label":label,"expiry":expiry})),
        )?;
        if token.secret.is_empty()
            || token.secret.len() > 8192
            || token.token.label != label
            || token.token.expiry_epoch()? == 0
        {
            return Err(RuntimeError::Incompatible(
                "invalid issued token metadata".to_owned(),
            ));
        }
        Ok(token)
    }
    fn list(&self, authorization: &ApiAuthorization) -> Result<Vec<TokenMetadata>, RuntimeError> {
        self.token_json(self.authorized_request(
            Method::GET,
            "@warpgate/api/profile/api-tokens",
            authorization,
        )?)
    }
    fn revoke(&self, authorization: &ApiAuthorization, id: &str) -> Result<(), RuntimeError> {
        self.require_managed_transport()?;
        if id.len() != 36
            || !id.bytes().enumerate().all(|(i, b)| {
                if matches!(i, 8 | 13 | 18 | 23) {
                    b == b'-'
                } else {
                    b.is_ascii_hexdigit()
                }
            })
        {
            return Err(RuntimeError::InvalidInput(
                "invalid managed token identifier".to_owned(),
            ));
        }
        let response = self
            .authorized_request(
                Method::DELETE,
                &format!("@warpgate/api/profile/api-tokens/{id}"),
                authorization,
            )?
            .send()?;
        if response.status() == StatusCode::NOT_FOUND {
            return Ok(());
        }
        checked(response)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{BufRead, BufReader, Write};
    use std::net::TcpListener;
    use std::sync::{Arc, Mutex};

    type Requests = Arc<Mutex<Vec<String>>>;
    fn server(
        responses: Vec<(u16, String, String)>,
    ) -> (String, Requests, std::thread::JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}/", listener.local_addr().unwrap());
        let requests = Arc::new(Mutex::new(Vec::new()));
        let captured = Arc::clone(&requests);
        let worker = std::thread::spawn(move || {
            for (status, headers, body) in responses {
                let (mut stream, _) = listener.accept().unwrap();
                stream
                    .set_read_timeout(Some(std::time::Duration::from_secs(3)))
                    .unwrap();
                let mut reader = BufReader::new(stream.try_clone().unwrap());
                let mut request = String::new();
                let mut length = 0;
                loop {
                    let mut line = String::new();
                    reader.read_line(&mut line).unwrap();
                    if line == "\r\n" || line.is_empty() {
                        break;
                    }
                    if let Some(value) = line.to_ascii_lowercase().strip_prefix("content-length:") {
                        length = value.trim().parse().unwrap();
                    }
                    request.push_str(&line);
                }
                let mut bytes = vec![0; length];
                reader.read_exact(&mut bytes).unwrap();
                request.push_str(std::str::from_utf8(&bytes).unwrap());
                captured.lock().unwrap().push(request);
                write!(stream, "HTTP/1.1 {status} Test\r\nContent-Length: {}\r\nContent-Type: application/json\r\n{headers}Connection: close\r\n\r\n{body}", body.len()).unwrap();
            }
        });
        (url, requests, worker)
    }
    #[test]
    fn official_token_contract_uses_origin_cookie_and_issued_secret() {
        let body = r#"{"token":{"id":"00000000-0000-0000-0000-000000000001","label":"owned-label","created":"2033-05-18T03:33:20Z","expiry":"2033-05-19T03:33:20Z"},"secret":"issued-secret"}"#;
        let (url, requests, worker) = server(vec![
            (201, String::new(), body.to_owned()),
            (204, String::new(), String::new()),
        ]);
        let client = ApiClient::new(&url).unwrap();
        let token = client
            .issue(
                &ApiAuthorization::Session("warpgate-http-session=test-cookie".to_owned()),
                "owned-label",
                2_000_086_400,
            )
            .unwrap();
        client
            .revoke(&ApiAuthorization::Token(token.secret), &token.token.id)
            .unwrap();
        worker.join().unwrap();
        let captured = requests.lock().unwrap();
        assert!(captured[0].starts_with("POST /@warpgate/api/profile/api-tokens HTTP/1.1"));
        assert!(
            captured[0]
                .to_ascii_lowercase()
                .contains("cookie: warpgate-http-session=test-cookie")
        );
        assert!(!captured[0].contains("x-warpgate-token"));
        assert!(captured[0].contains("\"label\":\"owned-label\""));
        assert!(captured[1].starts_with(
            "DELETE /@warpgate/api/profile/api-tokens/00000000-0000-0000-0000-000000000001"
        ));
        assert!(
            captured[1]
                .to_ascii_lowercase()
                .contains("x-warpgate-token: issued-secret")
        );
        assert!(!captured[1].contains("test-cookie"));
    }
    #[test]
    fn credentials_are_never_forwarded_to_a_redirect_target() {
        let target = TcpListener::bind("127.0.0.1:0").unwrap();
        target.set_nonblocking(true).unwrap();
        let redirect = format!(
            "Location: http://{}/stolen\r\n",
            target.local_addr().unwrap()
        );
        let (url, _, worker) = server(vec![(302, redirect, String::new())]);
        assert!(
            ApiClient::new(&url)
                .unwrap()
                .list(&ApiAuthorization::Token("secret".to_owned()))
                .is_err()
        );
        worker.join().unwrap();
        assert!(target.accept().is_err());
    }
    #[test]
    fn malformed_or_oversized_response_never_exposes_credentials() {
        for body in [
            "{\"secret\":\"do-not-report\"}".to_owned(),
            "x".repeat(65_537),
        ] {
            let (url, _, worker) = server(vec![(200, String::new(), body)]);
            let error = ApiClient::new(&url)
                .unwrap()
                .list(&ApiAuthorization::Token("request-secret".to_owned()))
                .unwrap_err();
            worker.join().unwrap();
            assert!(!error.to_string().contains("do-not-report"));
            assert!(!error.to_string().contains("request-secret"));
        }
        assert!(
            ApiClient::new("http://external.example/")
                .unwrap()
                .list(&ApiAuthorization::Token("secret".to_owned()))
                .is_err()
        );
    }
}
