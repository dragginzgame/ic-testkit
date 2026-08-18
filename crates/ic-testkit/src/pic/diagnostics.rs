use std::{
    fmt,
    panic::{AssertUnwindSafe, catch_unwind},
};

use candid::Principal;
use pocket_ic::{CanisterLogRecord, CanisterStatusResult, PocketIc, RejectResponse};

use super::transport;

/// Default maximum number of canister-log records retained in diagnostics.
pub const DEFAULT_CANISTER_LOG_RECORD_LIMIT: usize = 32;

/// Default maximum aggregate raw canister-log bytes retained in diagnostics.
pub const DEFAULT_CANISTER_LOG_BYTE_LIMIT: usize = 16 * 1024;

/// Explicit bounds applied when canister logs are converted to diagnostic text.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CanisterLogRenderLimits {
    record_limit: usize,
    byte_limit: usize,
}

impl CanisterLogRenderLimits {
    /// Set exact record and aggregate raw-content byte limits.
    ///
    /// Zero is valid for either limit and retains no content in that dimension.
    #[must_use]
    pub const fn new(record_limit: usize, byte_limit: usize) -> Self {
        Self {
            record_limit,
            byte_limit,
        }
    }

    /// Maximum number of retained records.
    #[must_use]
    pub const fn record_limit(self) -> usize {
        self.record_limit
    }

    /// Maximum aggregate number of retained raw content bytes.
    #[must_use]
    pub const fn byte_limit(self) -> usize {
        self.byte_limit
    }
}

impl Default for CanisterLogRenderLimits {
    fn default() -> Self {
        Self::new(
            DEFAULT_CANISTER_LOG_RECORD_LIMIT,
            DEFAULT_CANISTER_LOG_BYTE_LIMIT,
        )
    }
}

/// Exact controller-aware inputs for one best-effort diagnostic collection.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CanisterDiagnosticsRequest {
    canister_id: Principal,
    status_sender: Principal,
    log_sender: Principal,
    log_limits: CanisterLogRenderLimits,
}

impl CanisterDiagnosticsRequest {
    /// Create a request with independent, exact status and log senders.
    ///
    /// Anonymous access remains available only by explicitly supplying
    /// [`Principal::anonymous`] for the corresponding operation.
    #[must_use]
    pub fn new(canister_id: Principal, status_sender: Principal, log_sender: Principal) -> Self {
        Self {
            canister_id,
            status_sender,
            log_sender,
            log_limits: CanisterLogRenderLimits::default(),
        }
    }

    /// Override the bounds used to retain and render fetched log content.
    #[must_use]
    pub const fn with_log_limits(mut self, limits: CanisterLogRenderLimits) -> Self {
        self.log_limits = limits;
        self
    }

    /// Target canister.
    #[must_use]
    pub const fn canister_id(self) -> Principal {
        self.canister_id
    }

    /// Exact sender supplied to `canister_status`.
    #[must_use]
    pub const fn status_sender(self) -> Principal {
        self.status_sender
    }

    /// Exact sender supplied to `fetch_canister_logs`.
    #[must_use]
    pub const fn log_sender(self) -> Principal {
        self.log_sender
    }

    /// Log rendering bounds.
    #[must_use]
    pub const fn log_limits(self) -> CanisterLogRenderLimits {
        self.log_limits
    }
}

/// One caller-labeled exact diagnostic request in a collect-all batch.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LabeledCanisterDiagnosticsRequest {
    label: String,
    request: CanisterDiagnosticsRequest,
}

impl LabeledCanisterDiagnosticsRequest {
    /// Attach a stable caller-facing label to an exact diagnostic request.
    #[must_use]
    pub fn new(label: impl Into<String>, request: CanisterDiagnosticsRequest) -> Self {
        Self {
            label: label.into(),
            request,
        }
    }

    /// Caller-supplied label retained in the batch report.
    #[must_use]
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Exact controller-aware request.
    #[must_use]
    pub const fn request(&self) -> CanisterDiagnosticsRequest {
        self.request
    }

    /// Consume the labeled request into its caller label and exact request.
    #[must_use]
    pub fn into_parts(self) -> (String, CanisterDiagnosticsRequest) {
        (self.label, self.request)
    }
}

/// A failed best-effort PocketIC diagnostic call.
#[non_exhaustive]
#[derive(Debug)]
pub enum CanisterDiagnosticFailure {
    /// PocketIC returned a structured management-canister rejection.
    Rejected(RejectResponse),
    /// The PocketIC instance was no longer reachable.
    InstanceUnavailable {
        /// Captured transport or panic message.
        message: String,
    },
    /// PocketIC panicked for a reason other than a dead instance.
    Panicked {
        /// Captured panic message.
        message: String,
    },
}

impl fmt::Display for CanisterDiagnosticFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Rejected(response) => write!(formatter, "rejected: {response:?}"),
            Self::InstanceUnavailable { message } => {
                write!(formatter, "PocketIC instance unavailable: {message}")
            }
            Self::Panicked { message } => write!(formatter, "panicked: {message}"),
        }
    }
}

impl std::error::Error for CanisterDiagnosticFailure {}

/// One canister-log record retained as bounded lossy UTF-8 text.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CanisterDiagnosticLogRecord {
    index: u64,
    timestamp_nanos: u64,
    content: String,
    original_content_bytes: usize,
    omitted_content_bytes: usize,
}

impl CanisterDiagnosticLogRecord {
    /// Upstream record index.
    #[must_use]
    pub const fn index(&self) -> u64 {
        self.index
    }

    /// Upstream record timestamp in nanoseconds.
    #[must_use]
    pub const fn timestamp_nanos(&self) -> u64 {
        self.timestamp_nanos
    }

    /// Retained content converted with lossy UTF-8 decoding.
    #[must_use]
    pub fn content(&self) -> &str {
        &self.content
    }

    /// Original raw content length before bounding and conversion.
    #[must_use]
    pub const fn original_content_bytes(&self) -> usize {
        self.original_content_bytes
    }

    /// Raw content bytes omitted from this retained record.
    #[must_use]
    pub const fn omitted_content_bytes(&self) -> usize {
        self.omitted_content_bytes
    }

    /// Whether this record's content was truncated.
    #[must_use]
    pub const fn was_truncated(&self) -> bool {
        self.omitted_content_bytes != 0
    }
}

/// Successfully fetched canister logs after bounded text conversion.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CanisterDiagnosticLogs {
    records: Vec<CanisterDiagnosticLogRecord>,
    total_records: usize,
    total_content_bytes: usize,
    omitted_records: usize,
    omitted_content_bytes: usize,
}

impl CanisterDiagnosticLogs {
    /// Retained records in upstream order.
    #[must_use]
    pub fn records(&self) -> &[CanisterDiagnosticLogRecord] {
        &self.records
    }

    /// Total number of records returned by PocketIC before bounding.
    #[must_use]
    pub const fn total_records(&self) -> usize {
        self.total_records
    }

    /// Total raw content bytes returned by PocketIC before bounding.
    #[must_use]
    pub const fn total_content_bytes(&self) -> usize {
        self.total_content_bytes
    }

    /// Number of whole records omitted by the configured bounds.
    #[must_use]
    pub const fn omitted_records(&self) -> usize {
        self.omitted_records
    }

    /// Aggregate raw content bytes omitted across retained and omitted records.
    #[must_use]
    pub const fn omitted_content_bytes(&self) -> usize {
        self.omitted_content_bytes
    }

    /// Whether either whole records or record content were truncated.
    #[must_use]
    pub const fn was_truncated(&self) -> bool {
        self.omitted_records != 0 || self.omitted_content_bytes != 0
    }
}

impl fmt::Display for CanisterDiagnosticLogs {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.records.is_empty() {
            if self.total_records == 0 {
                formatter.write_str("<empty>")?;
            } else {
                formatter.write_str("<no retained records>")?;
            }
        } else {
            for (position, record) in self.records.iter().enumerate() {
                if position != 0 {
                    formatter.write_str(", ")?;
                }
                write!(
                    formatter,
                    "[{}@{}]={:?}",
                    record.index, record.timestamp_nanos, record.content
                )?;
                if record.was_truncated() {
                    write!(
                        formatter,
                        " (truncated {} bytes)",
                        record.omitted_content_bytes
                    )?;
                }
            }
        }
        if self.was_truncated() {
            write!(
                formatter,
                "; truncated omitted_records={} omitted_content_bytes={}",
                self.omitted_records, self.omitted_content_bytes
            )?;
        }
        Ok(())
    }
}

/// Independent status and log outcomes for one diagnostic collection.
#[derive(Debug)]
pub struct CanisterDiagnosticsReport {
    request: CanisterDiagnosticsRequest,
    status: Result<CanisterStatusResult, CanisterDiagnosticFailure>,
    logs: Result<CanisterDiagnosticLogs, CanisterDiagnosticFailure>,
}

impl CanisterDiagnosticsReport {
    /// Exact request used for this report.
    #[must_use]
    pub const fn request(&self) -> CanisterDiagnosticsRequest {
        self.request
    }

    /// Status result, independent of log retrieval.
    pub const fn status(&self) -> Result<&CanisterStatusResult, &CanisterDiagnosticFailure> {
        self.status.as_ref()
    }

    /// Log result, independent of status retrieval.
    pub const fn logs(&self) -> Result<&CanisterDiagnosticLogs, &CanisterDiagnosticFailure> {
        self.logs.as_ref()
    }

    /// Whether both status and log collection succeeded.
    #[must_use]
    pub const fn is_success(&self) -> bool {
        self.status.is_ok() && self.logs.is_ok()
    }

    /// Consume the report into its exact request and independent outcomes.
    pub fn into_parts(
        self,
    ) -> (
        CanisterDiagnosticsRequest,
        Result<CanisterStatusResult, CanisterDiagnosticFailure>,
        Result<CanisterDiagnosticLogs, CanisterDiagnosticFailure>,
    ) {
        (self.request, self.status, self.logs)
    }

    /// Render a compact, bounded diagnostic line suitable for failure output.
    #[must_use]
    pub fn render_compact(&self) -> String {
        self.to_string()
    }
}

/// One ordered labeled entry in a collect-all diagnostics batch.
#[derive(Debug)]
pub struct CanisterDiagnosticsBatchEntry {
    label: String,
    report: CanisterDiagnosticsReport,
}

impl CanisterDiagnosticsBatchEntry {
    /// Caller-supplied label.
    #[must_use]
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Structured status and log outcomes for this entry.
    #[must_use]
    pub const fn report(&self) -> &CanisterDiagnosticsReport {
        &self.report
    }

    /// Whether both status and log collection succeeded for this entry.
    #[must_use]
    pub const fn is_success(&self) -> bool {
        self.report.is_success()
    }

    /// Consume the entry into its caller label and structured report.
    #[must_use]
    pub fn into_parts(self) -> (String, CanisterDiagnosticsReport) {
        (self.label, self.report)
    }
}

/// Ordered entries from a sequential collect-all diagnostics batch.
#[derive(Debug, Default)]
pub struct CanisterDiagnosticsBatchReport {
    entries: Vec<CanisterDiagnosticsBatchEntry>,
}

impl CanisterDiagnosticsBatchReport {
    /// Entries in the supplied request order.
    #[must_use]
    pub fn entries(&self) -> &[CanisterDiagnosticsBatchEntry] {
        &self.entries
    }

    /// Entries with at least one failed status or log operation.
    pub fn failures(&self) -> impl Iterator<Item = &CanisterDiagnosticsBatchEntry> {
        self.entries.iter().filter(|entry| !entry.is_success())
    }

    /// Whether every entry collected both status and logs successfully.
    #[must_use]
    pub fn is_success(&self) -> bool {
        self.entries
            .iter()
            .all(CanisterDiagnosticsBatchEntry::is_success)
    }

    /// Consume the report into its ordered entries.
    #[must_use]
    pub fn into_entries(self) -> Vec<CanisterDiagnosticsBatchEntry> {
        self.entries
    }

    /// Render compact bounded diagnostics with retained caller labels.
    #[must_use]
    pub fn render_compact(&self) -> String {
        self.to_string()
    }
}

impl fmt::Display for CanisterDiagnosticsBatchReport {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "diagnostics={}", self.entries.len())?;
        for entry in &self.entries {
            write!(formatter, "; label={:?} {}", entry.label, entry.report)?;
        }
        Ok(())
    }
}

impl fmt::Display for CanisterDiagnosticsReport {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "canister={} status_sender={} status=",
            self.request.canister_id, self.request.status_sender
        )?;
        match &self.status {
            Ok(status) => write!(
                formatter,
                "ok(state={:?} version={} controllers={} module_hash_bytes={} memory_bytes={} cycles={})",
                status.status,
                status.version,
                status.settings.controllers.len(),
                status.module_hash.as_ref().map_or(0, Vec::len),
                status.memory_size,
                status.cycles,
            ),
            Err(failure) => write!(formatter, "<{failure}>"),
        }?;
        write!(formatter, " log_sender={} logs=", self.request.log_sender)?;
        match &self.logs {
            Err(failure) => write!(formatter, "<{failure}>")?,
            Ok(logs) => write!(formatter, "{logs}")?,
        }
        Ok(())
    }
}

/// Reusable structured PocketIC failure diagnostics.
pub trait PocketIcDiagnosticsExt {
    /// Collect status and bounded logs using the request's exact senders.
    ///
    /// Both calls are attempted independently. PocketIC transport panics are
    /// captured so diagnostics can remain subordinate to the original failure.
    fn collect_canister_diagnostics(
        &self,
        request: CanisterDiagnosticsRequest,
    ) -> CanisterDiagnosticsReport;

    /// Collect every labeled request sequentially in its supplied order.
    ///
    /// Every target is attempted even when an earlier entry fails or panics.
    /// Each entry preserves its exact request and independent status/log
    /// outcomes; no anonymous retry or fallback is performed.
    fn collect_canister_diagnostics_batch(
        &self,
        requests: &[LabeledCanisterDiagnosticsRequest],
    ) -> CanisterDiagnosticsBatchReport {
        let entries = requests
            .iter()
            .map(|labeled| {
                let request = labeled.request;
                let report = catch_unwind(AssertUnwindSafe(|| {
                    self.collect_canister_diagnostics(request)
                }))
                .unwrap_or_else(|payload| {
                    let message = transport::panic_payload_to_string(payload.as_ref());
                    CanisterDiagnosticsReport {
                        request,
                        status: Err(diagnostic_panic_failure(message.clone())),
                        logs: Err(diagnostic_panic_failure(message)),
                    }
                });
                CanisterDiagnosticsBatchEntry {
                    label: labeled.label.clone(),
                    report,
                }
            })
            .collect();
        CanisterDiagnosticsBatchReport { entries }
    }
}

impl PocketIcDiagnosticsExt for PocketIc {
    fn collect_canister_diagnostics(
        &self,
        request: CanisterDiagnosticsRequest,
    ) -> CanisterDiagnosticsReport {
        let status = capture_diagnostic_call(|| {
            self.canister_status(request.canister_id, Some(request.status_sender))
        });
        let logs = capture_diagnostic_call(|| {
            self.fetch_canister_logs(request.canister_id, request.log_sender)
        })
        .map(|records| render_log_records(records, request.log_limits));

        CanisterDiagnosticsReport {
            request,
            status,
            logs,
        }
    }
}

fn capture_diagnostic_call<T>(
    call: impl FnOnce() -> Result<T, RejectResponse>,
) -> Result<T, CanisterDiagnosticFailure> {
    match catch_unwind(AssertUnwindSafe(call)) {
        Ok(Ok(value)) => Ok(value),
        Ok(Err(response)) => Err(CanisterDiagnosticFailure::Rejected(response)),
        Err(payload) => {
            let message = transport::panic_payload_to_string(payload.as_ref());
            Err(diagnostic_panic_failure(message))
        }
    }
}

fn diagnostic_panic_failure(message: String) -> CanisterDiagnosticFailure {
    if transport::is_dead_instance_transport_error(&message) {
        CanisterDiagnosticFailure::InstanceUnavailable { message }
    } else {
        CanisterDiagnosticFailure::Panicked { message }
    }
}

fn render_log_records(
    records: Vec<CanisterLogRecord>,
    limits: CanisterLogRenderLimits,
) -> CanisterDiagnosticLogs {
    let total_records = records.len();
    let total_content_bytes = records.iter().fold(0usize, |total, record| {
        total.saturating_add(record.content.len())
    });
    let mut rendered = Vec::with_capacity(total_records.min(limits.record_limit));
    let mut retained_bytes = 0usize;
    let mut omitted_records = 0usize;
    let mut omitted_content_bytes = 0usize;

    for record in records {
        if rendered.len() == limits.record_limit || retained_bytes == limits.byte_limit {
            omitted_records = omitted_records.saturating_add(1);
            omitted_content_bytes = omitted_content_bytes.saturating_add(record.content.len());
            continue;
        }

        let available = limits.byte_limit.saturating_sub(retained_bytes);
        let retained = record.content.len().min(available);
        let omitted = record.content.len().saturating_sub(retained);
        let content = String::from_utf8_lossy(&record.content[..retained]).into_owned();
        retained_bytes = retained_bytes.saturating_add(retained);
        omitted_content_bytes = omitted_content_bytes.saturating_add(omitted);
        rendered.push(CanisterDiagnosticLogRecord {
            index: record.idx,
            timestamp_nanos: record.timestamp_nanos,
            content,
            original_content_bytes: record.content.len(),
            omitted_content_bytes: omitted,
        });
    }

    CanisterDiagnosticLogs {
        records: rendered,
        total_records,
        total_content_bytes,
        omitted_records,
        omitted_content_bytes,
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use candid::Principal;
    use pocket_ic::CanisterLogRecord;

    use super::{
        CanisterDiagnosticFailure, CanisterDiagnosticsReport, CanisterDiagnosticsRequest,
        CanisterLogRenderLimits, LabeledCanisterDiagnosticsRequest, PocketIcDiagnosticsExt,
        render_log_records,
    };

    struct PanickingThenReporting {
        calls: Cell<usize>,
    }

    impl PocketIcDiagnosticsExt for PanickingThenReporting {
        fn collect_canister_diagnostics(
            &self,
            request: CanisterDiagnosticsRequest,
        ) -> CanisterDiagnosticsReport {
            let call = self.calls.get();
            self.calls.set(call + 1);
            assert_ne!(call, 0, "synthetic first-entry diagnostic panic");
            CanisterDiagnosticsReport {
                request,
                status: Err(CanisterDiagnosticFailure::Panicked {
                    message: "synthetic status failure".to_owned(),
                }),
                logs: Err(CanisterDiagnosticFailure::Panicked {
                    message: "synthetic log failure".to_owned(),
                }),
            }
        }
    }

    #[test]
    fn labeled_batch_retains_order_and_continues_after_entry_panic() {
        let collector = PanickingThenReporting {
            calls: Cell::new(0),
        };
        let first = CanisterDiagnosticsRequest::new(
            Principal::from_slice(&[1]),
            Principal::from_slice(&[2]),
            Principal::from_slice(&[3]),
        );
        let second = CanisterDiagnosticsRequest::new(
            Principal::from_slice(&[4]),
            Principal::from_slice(&[5]),
            Principal::from_slice(&[6]),
        );
        let report = collector.collect_canister_diagnostics_batch(&[
            LabeledCanisterDiagnosticsRequest::new("root", first),
            LabeledCanisterDiagnosticsRequest::new("worker", second),
        ]);

        assert_eq!(collector.calls.get(), 2);
        assert_eq!(report.entries().len(), 2);
        assert_eq!(report.entries()[0].label(), "root");
        assert_eq!(report.entries()[0].report().request(), first);
        assert_eq!(report.entries()[1].label(), "worker");
        assert_eq!(report.entries()[1].report().request(), second);
        assert_eq!(report.failures().count(), 2);
        assert!(!report.is_success());
        let compact = report.render_compact();
        assert!(compact.contains("label=\"root\""));
        assert!(compact.contains("label=\"worker\""));
        assert!(compact.contains("synthetic first-entry diagnostic panic"));
    }

    #[test]
    fn log_rendering_is_bounded_lossy_utf8_and_reports_truncation() {
        let logs = render_log_records(
            vec![
                CanisterLogRecord {
                    idx: 7,
                    timestamp_nanos: 11,
                    content: vec![b'f', 0x80, b'o'],
                },
                CanisterLogRecord {
                    idx: 8,
                    timestamp_nanos: 12,
                    content: b"bar".to_vec(),
                },
            ],
            CanisterLogRenderLimits::new(1, 2),
        );

        assert_eq!(logs.total_records(), 2);
        assert_eq!(logs.total_content_bytes(), 6);
        assert_eq!(logs.omitted_records(), 1);
        assert_eq!(logs.omitted_content_bytes(), 4);
        assert!(logs.was_truncated());
        assert_eq!(logs.records().len(), 1);
        assert_eq!(logs.records()[0].content(), "f�");
        assert_eq!(logs.records()[0].original_content_bytes(), 3);
        assert_eq!(logs.records()[0].omitted_content_bytes(), 1);
        assert!(logs.records()[0].was_truncated());
        let rendered = logs.to_string();
        assert!(rendered.contains("f�"));
        assert!(rendered.contains("truncated omitted_records=1 omitted_content_bytes=4"));
    }

    #[test]
    fn zero_log_bounds_retain_only_aggregate_truncation() {
        let logs = render_log_records(
            vec![CanisterLogRecord {
                idx: 1,
                timestamp_nanos: 2,
                content: b"hello".to_vec(),
            }],
            CanisterLogRenderLimits::new(0, 0),
        );

        assert!(logs.records().is_empty());
        assert_eq!(logs.omitted_records(), 1);
        assert_eq!(logs.omitted_content_bytes(), 5);
        assert!(logs.was_truncated());
        assert_eq!(
            logs.to_string(),
            "<no retained records>; truncated omitted_records=1 omitted_content_bytes=5"
        );
    }
}
