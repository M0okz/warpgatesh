use std::process::ExitCode;

use warpgatesh_core::schedule::SyncSchedule;

fn main() -> ExitCode {
    let schedule = SyncSchedule::default();

    match std::env::args().nth(1).as_deref() {
        Some("--help" | "-h") => {
            println!("Usage: warpgatesh-agent [--once]");
            ExitCode::SUCCESS
        }
        Some("--once") => {
            println!("warpgatesh-agent: synchronization transport is not implemented yet");
            ExitCode::from(2)
        }
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
