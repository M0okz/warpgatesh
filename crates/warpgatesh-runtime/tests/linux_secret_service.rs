#![cfg(target_os = "linux")]

use std::time::{SystemTime, UNIX_EPOCH};

use warpgatesh_runtime::keychain::{SystemKeychain, TokenStore};

#[test]
fn stores_reads_and_deletes_a_token_in_secret_service() {
    if std::env::var_os("WARPGATESH_TEST_SECRET_SERVICE").is_none() {
        eprintln!("skipped: set WARPGATESH_TEST_SECRET_SERVICE=1 to test a live Secret Service");
        return;
    }

    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time")
        .as_nanos();
    let profile = format!("integration-test-{nonce}");
    let store = SystemKeychain;
    let token = "warpgatesh-integration-test-token";

    store.set(&profile, token).expect("store test token");
    let retrieved = store.get(&profile).expect("read test token");
    store.delete(&profile).expect("delete test token");

    assert_eq!(retrieved, token);
    assert!(store.get(&profile).is_err(), "deleted token must be absent");
}
