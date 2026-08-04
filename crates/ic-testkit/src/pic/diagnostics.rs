use candid::Principal;
use pocket_ic::PocketIc;

use super::startup;

/// Reusable PocketIC failure diagnostics.
pub trait PocketIcDiagnosticsExt {
    /// Print status and canister-log context for a failed test operation.
    fn dump_canister_debug(&self, canister_id: Principal, context: &str);
}

impl PocketIcDiagnosticsExt for PocketIc {
    /// Dump basic PocketIC status and log context for one canister.
    fn dump_canister_debug(&self, canister_id: Principal, context: &str) {
        eprintln!("{context}: debug for canister {canister_id}");

        match self.canister_status(canister_id, None) {
            Ok(status) => eprintln!("canister_status: {status:?}"),
            Err(err) => {
                let message = err.to_string();
                if startup::is_dead_instance_transport_error(&message) {
                    eprintln!("canister_status unavailable: PocketIC instance no longer reachable");
                    return;
                }
                eprintln!("canister_status failed: {err:?}");
            }
        }

        match self.fetch_canister_logs(canister_id, Principal::anonymous()) {
            Ok(records) => {
                if records.is_empty() {
                    eprintln!("canister logs: <empty>");
                } else {
                    for record in records {
                        eprintln!("canister log: {record:?}");
                    }
                }
            }
            Err(err) => {
                let message = err.to_string();
                if startup::is_dead_instance_transport_error(&message) {
                    eprintln!(
                        "fetch_canister_logs unavailable: PocketIC instance no longer reachable"
                    );
                    return;
                }
                eprintln!("fetch_canister_logs failed: {err:?}");
            }
        }
    }
}
