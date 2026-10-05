//! Opt-in fixture benchmark driver for native Linux and macOS hosts.
//! Builds and measures the existing workload; library APIs own no benchmark policy.

mod fixture_reuse_driver;

use std::process::ExitCode;

use fixture_reuse_driver::Error;

fn main() -> ExitCode {
    match fixture_reuse_driver::run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            match error {
                Error::InvalidArgument(_) => ExitCode::from(2),
                Error::Interrupted => ExitCode::from(130),
                _ => ExitCode::FAILURE,
            }
        }
    }
}
