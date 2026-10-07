mod support;

use candid::Principal;
use ic_testkit::pic::{
    CanisterDiagnosticFailure, CanisterDiagnosticsRequest, LabeledCanisterDiagnosticsRequest,
    PocketIcDiagnosticsExt, is_dead_pocket_ic_transport_error,
};
use pocket_ic::CanisterSettings;

#[test]
fn diagnostics_use_independent_exact_senders_and_preserve_both_outcomes() {
    let pocket_ic = support::pocket_ic();
    let status_sender = Principal::from_slice(&[41]);
    let log_sender = Principal::from_slice(&[42]);
    let outsider = Principal::from_slice(&[43]);
    let canister_id = pocket_ic.create_canister_with_settings(
        None,
        Some(CanisterSettings {
            controllers: Some(vec![status_sender, log_sender]),
            ..CanisterSettings::default()
        }),
    );

    let request = CanisterDiagnosticsRequest::new(canister_id, status_sender, log_sender);
    let report = pocket_ic.collect_canister_diagnostics(request);

    assert_eq!(report.request(), request);
    assert!(report.status().is_ok(), "{report}");
    assert!(report.logs().is_ok(), "{report}");
    let compact = report.render_compact();
    assert!(compact.contains("status=ok(state="));
    assert!(compact.contains("logs=<empty>"));

    let batch = pocket_ic
        .collect_canister_diagnostics_batch(&[
            LabeledCanisterDiagnosticsRequest::new(
                "denied",
                CanisterDiagnosticsRequest::new(canister_id, outsider, outsider),
            ),
            LabeledCanisterDiagnosticsRequest::new("controller", request),
        ])
        .expect("valid labeled diagnostics batch");
    assert_eq!(batch.entries().len(), 2);
    assert_eq!(batch.entries()[0].label(), "denied");
    assert_eq!(batch.entries()[1].label(), "controller");
    assert!(
        batch
            .entries()
            .iter()
            .all(|entry| entry.entry_elapsed() <= batch.total())
    );
    let denied = batch.entries()[0].report();
    assert!(matches!(
        denied.status(),
        Err(CanisterDiagnosticFailure::Rejected(_))
    ));
    assert!(matches!(
        denied.logs(),
        Err(CanisterDiagnosticFailure::Rejected(_))
    ));
    for error in [denied.status().unwrap_err(), denied.logs().unwrap_err()] {
        assert!(
            !is_dead_pocket_ic_transport_error(error),
            "controller rejection must not be interpreted as a dead transport",
        );
    }
    assert!(batch.entries()[1].is_success());
    assert_eq!(batch.failures().count(), 1);
}
