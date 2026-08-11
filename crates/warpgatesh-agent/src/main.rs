use std::process::ExitCode;

use warpgatesh_core::schedule::SyncSchedule;
use warpgatesh_runtime::keychain::SystemKeychain;
use warpgatesh_runtime::storage::LocalStore;
use warpgatesh_runtime::sync::synchronize_all;

fn main() -> ExitCode {
    let schedule = SyncSchedule::default();

    match std::env::args().nth(1).as_deref() {
        Some("--help" | "-h") => {
            println!("Usage: warpgatesh-agent [--once]");
            ExitCode::SUCCESS
        }
        Some("--once") => match synchronize_once() {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("warpgatesh-agent: {error}");
                ExitCode::FAILURE
            }
        },
        Some(argument) => {
            eprintln!("warpgatesh-agent: unknown argument '{argument}'");
            ExitCode::from(2)
        }
        None => {
            println!(
                "warpgatesh-agent: ready (default interval: {} seconds)",
                schedule.interval.as_secs()
            );
            ExitCode::SUCCESS
        }
    }
}

fn synchronize_once() -> Result<(), warpgatesh_runtime::RuntimeError> {
    let store = LocalStore::for_current_user()?;
    let report = synchronize_all(&store, &SystemKeychain)?;
    println!(
        "Synchronized {} SSH target(s) from {} profile(s): +{}, -{}",
        report.target_count, report.profile_count, report.added, report.removed
    );
    Ok(())
}
