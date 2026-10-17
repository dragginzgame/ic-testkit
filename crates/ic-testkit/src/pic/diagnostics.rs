use std::{
    collections::HashMap,
    fmt, io,
    time::{Duration, Instant},
};

use candid::Principal;
use pocket_ic::{CanisterLogRecord, CanisterStatusResult, PocketIc};

use super::{
    PocketIcCallFailure,
    transport::{self, capture_call},
};

/// Default maximum number of canister-log records retained in diagnostics.
pub const DEFAULT_CANISTER_LOG_RECORD_LIMIT: usize = 32;

/// Default maximum aggregate raw canister-log bytes retained in diagnostics.
pub const DEFAULT_CANISTER_LOG_BYTE_LIMIT: usize = 16 * 1024;

/// Explicit bounds applied when canister logs are converted to diagnostic text.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CanisterLogRenderLimits {
    /// Maximum retained records; zero retains none.
    pub record_limit: usize,
    /// Maximum retained raw-content bytes; zero retains none.
    pub byte_limit: usize,
}

impl Default for CanisterLogRenderLimits {
    fn default() -> Self {
        Self {
            record_limit: DEFAULT_CANISTER_LOG_RECORD_LIMIT,
            byte_limit: DEFAULT_CANISTER_LOG_BYTE_LIMIT,
        }
    }
}

/// Exact controller-aware inputs for one best-effort diagnostic collection.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CanisterDiagnosticsRequest {
    /// Canister to inspect.
    pub canister_id: Principal,
    /// Exact sender for the status request.
    pub status_sender: Principal,
    /// Exact sender for the log request.
    pub log_sender: Principal,
    /// Caller-selected log rendering bounds.
    pub log_limits: CanisterLogRenderLimits,
}

/// One caller-labeled exact diagnostic request in a collect-all batch.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LabeledCanisterDiagnosticsRequest {
    /// Caller label, checked for emptiness and uniqueness before collection.
    pub label: String,
    /// Exact controller-aware diagnostic inputs.
    pub request: CanisterDiagnosticsRequest,
}

/// Structural error that prevents a diagnostics batch from starting.
#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CanisterDiagnosticsBatchContractError {
    /// An entry label was empty.
    EmptyLabel {
        /// Zero-based position of the invalid entry.
        index: usize,
    },
    /// Two entries used the same label.
    DuplicateLabel {
        /// Duplicated caller label.
        label: String,
        /// Position where the label first appeared.
        first_index: usize,
        /// Position where the label was repeated.
        duplicate_index: usize,
    },
}

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
        self.total_records - self.records.len()
    }

    /// Aggregate raw content bytes omitted across retained and omitted records.
    #[must_use]
    pub const fn omitted_content_bytes(&self) -> usize {
        self.omitted_content_bytes
    }

    /// Whether either whole records or record content were truncated.
    #[must_use]
    pub const fn was_truncated(&self) -> bool {
        self.omitted_records() != 0 || self.omitted_content_bytes != 0
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
                self.omitted_records(),
                self.omitted_content_bytes
            )?;
        }
        Ok(())
    }
}

/// Independent status and log outcomes for one diagnostic collection.
#[derive(Debug)]
pub struct CanisterDiagnosticsReport {
    request: CanisterDiagnosticsRequest,
    status: Result<CanisterStatusResult, PocketIcCallFailure>,
    logs: Result<CanisterDiagnosticLogs, PocketIcCallFailure>,
}

impl CanisterDiagnosticsReport {
    /// Exact request used for this report.
    #[must_use]
    pub const fn request(&self) -> CanisterDiagnosticsRequest {
        self.request
    }

    /// Status result, independent of log retrieval.
    pub const fn status(&self) -> Result<&CanisterStatusResult, &PocketIcCallFailure> {
        self.status.as_ref()
    }

    /// Log result, independent of status retrieval.
    pub const fn logs(&self) -> Result<&CanisterDiagnosticLogs, &PocketIcCallFailure> {
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
        Result<CanisterStatusResult, PocketIcCallFailure>,
        Result<CanisterDiagnosticLogs, PocketIcCallFailure>,
    ) {
        (self.request, self.status, self.logs)
    }
}

/// One ordered labeled entry in a collect-all diagnostics batch.
#[derive(Debug)]
pub struct CanisterDiagnosticsBatchEntry {
    label: String,
    report: CanisterDiagnosticsReport,
    entry_elapsed: Duration,
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

    /// Complete wall-clock time for this target's diagnostic collection.
    #[must_use]
    pub const fn entry_elapsed(&self) -> Duration {
        self.entry_elapsed
    }

    /// Consume the entry into its label, structured report, and wall time.
    #[must_use]
    pub fn into_parts(self) -> (String, CanisterDiagnosticsReport, Duration) {
        (self.label, self.report, self.entry_elapsed)
    }
}

/// Ordered entries from a sequential collect-all diagnostics batch.
#[derive(Debug, Default)]
pub struct CanisterDiagnosticsBatchReport {
    entries: Vec<CanisterDiagnosticsBatchEntry>,
    total: Duration,
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

    /// Complete wall-clock time for the sequential collect-all batch.
    #[must_use]
    pub const fn total(&self) -> Duration {
        self.total
    }

    /// Consume the report into its ordered entries.
    #[must_use]
    pub fn into_entries(self) -> Vec<CanisterDiagnosticsBatchEntry> {
        self.entries
    }
}

impl fmt::Display for CanisterDiagnosticsBatchReport {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "diagnostics={} total={:?}",
            self.entries.len(),
            self.total
        )?;
        for entry in &self.entries {
            write!(
                formatter,
                "; label={:?} elapsed={:?} {}",
                entry.label, entry.entry_elapsed, entry.report
            )?;
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

    /// Explicitly collect and write one contextual diagnostic report.
    ///
    /// Uses the request's exact senders and log limits, with no retry or output
    /// destination selected implicitly. Collection, formatting and writer panics
    /// are captured as an IO error retaining [`super::PocketIcOperationError`].
    /// Ordinary writer errors are returned unchanged. Status/log failures remain
    /// in the written report; successful output does not mean those calls passed.
    /// Callers keep their primary failure authoritative and decide whether to
    /// retain or ignore this secondary reporting result.
    fn write_canister_diagnostics(
        &self,
        request: CanisterDiagnosticsRequest,
        context: &str,
        writer: &mut dyn io::Write,
    ) -> io::Result<()> {
        transport::capture_operation(|| {
            let report = self.collect_canister_diagnostics(request);
            writeln!(writer, "{context}: {report}")
        })
        .map_err(io::Error::other)?
    }

    /// Collect every labeled request sequentially in its supplied order.
    ///
    /// Every target is attempted even when an earlier entry fails or panics.
    /// Each entry preserves its exact request and independent status/log
    /// outcomes; no anonymous retry or fallback is performed.
    fn collect_canister_diagnostics_batch(
        &self,
        requests: &[LabeledCanisterDiagnosticsRequest],
    ) -> Result<CanisterDiagnosticsBatchReport, CanisterDiagnosticsBatchContractError> {
        validate_diagnostics_batch_labels(requests)?;
        let started = Instant::now();
        let entries = requests
            .iter()
            .map(|labeled| {
                let entry_started = Instant::now();
                let request = labeled.request;
                let report =
                    transport::capture_operation(|| self.collect_canister_diagnostics(request))
                        .unwrap_or_else(|source| CanisterDiagnosticsReport {
                            request,
                            status: Err(PocketIcCallFailure::Operation(source.clone())),
                            logs: Err(PocketIcCallFailure::Operation(source)),
                        });
                CanisterDiagnosticsBatchEntry {
                    label: labeled.label.clone(),
                    report,
                    entry_elapsed: entry_started.elapsed(),
                }
            })
            .collect();
        Ok(CanisterDiagnosticsBatchReport {
            entries,
            total: started.elapsed(),
        })
    }
}

fn validate_diagnostics_batch_labels(
    requests: &[LabeledCanisterDiagnosticsRequest],
) -> Result<(), CanisterDiagnosticsBatchContractError> {
    let mut seen = HashMap::new();
    for (index, request) in requests.iter().enumerate() {
        let label = request.label.as_str();
        if label.is_empty() {
            return Err(CanisterDiagnosticsBatchContractError::EmptyLabel { index });
        }
        if let Some(first_index) = seen.insert(label, index) {
            return Err(CanisterDiagnosticsBatchContractError::DuplicateLabel {
                label: label.to_owned(),
                first_index,
                duplicate_index: index,
            });
        }
    }
    Ok(())
}

impl fmt::Display for CanisterDiagnosticsBatchContractError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyLabel { index } => {
                write!(
                    formatter,
                    "diagnostics batch label at index {index} is empty"
                )
            }
            Self::DuplicateLabel {
                label,
                first_index,
                duplicate_index,
            } => write!(
                formatter,
                "diagnostics batch label {label:?} at index {duplicate_index} duplicates index {first_index}",
            ),
        }
    }
}

impl std::error::Error for CanisterDiagnosticsBatchContractError {}

impl PocketIcDiagnosticsExt for PocketIc {
    fn collect_canister_diagnostics(
        &self,
        request: CanisterDiagnosticsRequest,
    ) -> CanisterDiagnosticsReport {
        let status =
            capture_call(|| self.canister_status(request.canister_id, Some(request.status_sender)));
        let logs =
            capture_call(|| self.fetch_canister_logs(request.canister_id, request.log_sender))
                .map(|records| render_log_records(records, request.log_limits));

        CanisterDiagnosticsReport {
            request,
            status,
            logs,
        }
    }
}

fn render_log_records(
    records: Vec<CanisterLogRecord>,
    limits: CanisterLogRenderLimits,
) -> CanisterDiagnosticLogs {
    let total_records = records.len();
    let mut total_content_bytes = 0usize;
    let mut rendered = Vec::with_capacity(total_records.min(limits.record_limit));
    let mut retained_bytes = 0usize;
    let mut omitted_content_bytes = 0usize;

    for record in records {
        total_content_bytes = total_content_bytes.saturating_add(record.content.len());
        if rendered.len() == limits.record_limit || retained_bytes == limits.byte_limit {
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
        omitted_content_bytes,
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use candid::Principal;
    use pocket_ic::CanisterLogRecord;

    use super::{
        CanisterDiagnosticsBatchContractError, CanisterDiagnosticsReport,
        CanisterDiagnosticsRequest, CanisterLogRenderLimits, LabeledCanisterDiagnosticsRequest,
        PocketIcCallFailure, PocketIcDiagnosticsExt, capture_call, render_log_records,
    };

    #[test]
    fn diagnostic_transport_classification_survives_context_and_io_wrapping() {
        const REFUSED: &str = "HTTP failure: reqwest::Error { kind: Request, url: \"http://127.0.0.1:1234/instances/0/read/get_time\", source: ConnectError(\"tcp connect error\", Os { code: 111, kind: ConnectionRefused, message: \"Connection refused\" }) }";
        const RESET: &str = "HTTP failure: reqwest::Error { kind: Request, url: \"http://127.0.0.1:49238/instances/0/read/get_time\", source: hyper_util::client::legacy::Error(SendRequest, hyper::Error(Io, Os { code: 54, kind: ConnectionReset, message: \"Connection reset by peer\" })) }";
        for message in [REFUSED, RESET, "application worker channel closed"] {
            let error = capture_call::<()>(|| panic!("{message}"))
                .expect_err("diagnostic operation should capture the panic");
            let expected = message != "application worker channel closed";
            let PocketIcCallFailure::Operation(source) = &error else {
                panic!("expected the original operation error");
            };
            assert_eq!(source.message(), message);
            assert_eq!(source.is_transport(), expected);
            let chained = std::error::Error::source(&error)
                .and_then(|cause| cause.downcast_ref::<super::super::PocketIcOperationError>())
                .expect("diagnostic error exposes its original typed cause");
            assert_eq!(chained, source);
            assert_eq!(
                super::super::is_dead_pocket_ic_transport_error(&error),
                expected,
            );
            let wrapped = std::io::Error::other(error);
            assert_eq!(
                super::super::is_dead_pocket_ic_transport_error(&wrapped),
                expected,
            );
        }
    }

    struct PanickingThenReporting {
        calls: Cell<usize>,
        request: Cell<Option<CanisterDiagnosticsRequest>>,
    }

    impl PocketIcDiagnosticsExt for PanickingThenReporting {
        fn collect_canister_diagnostics(
            &self,
            request: CanisterDiagnosticsRequest,
        ) -> CanisterDiagnosticsReport {
            let call = self.calls.get();
            self.calls.set(call + 1);
            self.request.set(Some(request));
            assert_ne!(call, 0, "synthetic first-entry diagnostic panic");
            CanisterDiagnosticsReport {
                request,
                status: Err(PocketIcCallFailure::Operation(
                    super::super::PocketIcOperationError::new("synthetic status failure"),
                )),
                logs: Err(PocketIcCallFailure::Operation(
                    super::super::PocketIcOperationError::new("synthetic log failure"),
                )),
            }
        }
    }

    struct PanickingWriter;

    impl std::io::Write for PanickingWriter {
        fn write(&mut self, _: &[u8]) -> std::io::Result<usize> {
            panic!("synthetic writer panic");
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn explicit_reporting_preserves_requests_and_contains_secondary_failures() {
        let collector = PanickingThenReporting {
            calls: Cell::new(0),
            request: Cell::new(None),
        };
        let request = CanisterDiagnosticsRequest {
            canister_id: Principal::from_slice(&[1]),
            status_sender: Principal::from_slice(&[2]),
            log_sender: Principal::from_slice(&[3]),
            log_limits: CanisterLogRenderLimits::default(),
        };
        let mut output = Vec::new();
        let error = collector
            .write_canister_diagnostics(request, "install", &mut output)
            .expect_err("collection panic becomes secondary error");
        assert_eq!(output, [] as [u8; 0]);
        assert_eq!(collector.calls.get(), 1);
        assert_eq!(collector.request.get(), Some(request));
        assert!(
            error
                .get_ref()
                .unwrap()
                .downcast_ref::<super::super::PocketIcOperationError>()
                .unwrap()
                .message()
                .contains("synthetic first-entry diagnostic panic")
        );

        collector
            .write_canister_diagnostics(request, "install", &mut output)
            .unwrap();
        let text = String::from_utf8(output).unwrap();
        assert!(text.starts_with("install: "));
        assert!(
            text.contains("synthetic status failure") && text.contains("synthetic log failure")
        );
        assert_eq!(collector.calls.get(), 2);
        assert_eq!(collector.request.get(), Some(request));

        let mut full_writer = std::io::Cursor::new([0_u8; 0]);
        let error = collector
            .write_canister_diagnostics(request, "install", &mut full_writer)
            .expect_err("ordinary writer failure is retained");
        assert_eq!(error.kind(), std::io::ErrorKind::WriteZero);
        let error = collector
            .write_canister_diagnostics(request, "install", &mut PanickingWriter)
            .expect_err("writer panic becomes secondary error");
        assert!(
            error
                .get_ref()
                .unwrap()
                .downcast_ref::<super::super::PocketIcOperationError>()
                .unwrap()
                .message()
                .contains("synthetic writer panic")
        );
        assert_eq!(
            collector.calls.get(),
            4,
            "one explicit collection per call, with no retries"
        );
    }

    #[test]
    fn labeled_batch_retains_order_and_continues_after_entry_panic() {
        let collector = PanickingThenReporting {
            calls: Cell::new(0),
            request: Cell::new(None),
        };
        let first = CanisterDiagnosticsRequest {
            canister_id: Principal::from_slice(&[1]),
            status_sender: Principal::from_slice(&[2]),
            log_sender: Principal::from_slice(&[3]),
            log_limits: CanisterLogRenderLimits::default(),
        };
        let second = CanisterDiagnosticsRequest {
            canister_id: Principal::from_slice(&[4]),
            status_sender: Principal::from_slice(&[5]),
            log_sender: Principal::from_slice(&[6]),
            log_limits: CanisterLogRenderLimits::default(),
        };
        let report = collector
            .collect_canister_diagnostics_batch(&[
                LabeledCanisterDiagnosticsRequest {
                    label: "root".into(),
                    request: first,
                },
                LabeledCanisterDiagnosticsRequest {
                    label: "worker".into(),
                    request: second,
                },
            ])
            .expect("valid labeled diagnostics batch");

        assert_eq!(collector.calls.get(), 2);
        assert_eq!(report.entries().len(), 2);
        assert_eq!(report.entries()[0].label(), "root");
        assert_eq!(report.entries()[0].report().request(), first);
        assert_eq!(report.entries()[1].label(), "worker");
        assert_eq!(report.entries()[1].report().request(), second);
        assert!(
            report
                .entries()
                .iter()
                .all(|entry| entry.entry_elapsed() <= report.total())
        );
        assert_eq!(report.failures().count(), 2);
        assert!(!report.is_success());
        let compact = report.to_string();
        assert!(compact.contains("label=\"root\""));
        assert!(compact.contains("label=\"worker\""));
        assert!(compact.contains("synthetic first-entry diagnostic panic"));
    }

    #[test]
    fn diagnostic_batch_rejects_invalid_labels_before_collection() {
        let collector = PanickingThenReporting {
            calls: Cell::new(0),
            request: Cell::new(None),
        };
        let request = CanisterDiagnosticsRequest {
            canister_id: Principal::from_slice(&[1]),
            status_sender: Principal::from_slice(&[2]),
            log_sender: Principal::from_slice(&[3]),
            log_limits: CanisterLogRenderLimits::default(),
        };
        let empty = collector
            .collect_canister_diagnostics_batch(&[LabeledCanisterDiagnosticsRequest {
                label: String::new(),
                request,
            }])
            .expect_err("empty label must reject diagnostics batch");
        assert_eq!(
            empty,
            CanisterDiagnosticsBatchContractError::EmptyLabel { index: 0 }
        );
        assert_eq!(collector.calls.get(), 0);

        let duplicate = collector
            .collect_canister_diagnostics_batch(&[
                LabeledCanisterDiagnosticsRequest {
                    label: "same".into(),
                    request,
                },
                LabeledCanisterDiagnosticsRequest {
                    label: "same".into(),
                    request,
                },
            ])
            .expect_err("duplicate labels must reject diagnostics batch");
        assert_eq!(
            duplicate,
            CanisterDiagnosticsBatchContractError::DuplicateLabel {
                label: "same".to_owned(),
                first_index: 0,
                duplicate_index: 1,
            }
        );
        assert_eq!(collector.calls.get(), 0);
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
            CanisterLogRenderLimits {
                record_limit: 1,
                byte_limit: 2,
            },
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
        for (record_limit, byte_limit) in [(0, 0), (0, 16), (16, 0)] {
            let logs = render_log_records(
                vec![CanisterLogRecord {
                    idx: 1,
                    timestamp_nanos: 2,
                    content: b"hello".to_vec(),
                }],
                CanisterLogRenderLimits {
                    record_limit,
                    byte_limit,
                },
            );

            assert_eq!(logs.records(), []);
            assert_eq!(logs.total_records(), 1);
            assert_eq!(logs.total_content_bytes(), 5);
            assert_eq!(logs.omitted_records(), 1);
            assert_eq!(logs.omitted_content_bytes(), 5);
            assert!(logs.was_truncated());
            assert_eq!(
                logs.to_string(),
                "<no retained records>; truncated omitted_records=1 omitted_content_bytes=5"
            );
        }
    }

    #[test]
    fn logs_within_bounds_preserve_empty_records_and_raw_byte_totals() {
        let limits = CanisterLogRenderLimits {
            record_limit: 3,
            byte_limit: 3,
        };
        let empty = render_log_records(vec![], limits);
        assert_eq!(empty.total_records(), 0);
        assert_eq!(empty.total_content_bytes(), 0);
        assert_eq!(empty.omitted_records(), 0);
        assert_eq!(empty.omitted_content_bytes(), 0);
        assert!(!empty.was_truncated());
        assert_eq!(empty.to_string(), "<empty>");

        let logs = render_log_records(
            vec![
                CanisterLogRecord {
                    idx: 1,
                    timestamp_nanos: 2,
                    content: vec![],
                },
                CanisterLogRecord {
                    idx: 3,
                    timestamp_nanos: 4,
                    content: vec![0x80, b'a'],
                },
                CanisterLogRecord {
                    idx: 5,
                    timestamp_nanos: 6,
                    content: vec![],
                },
            ],
            limits,
        );
        assert_eq!(logs.total_records(), 3);
        assert_eq!(logs.records().len(), 3);
        assert_eq!(logs.total_content_bytes(), 2);
        assert_eq!(logs.records()[0].content(), "");
        assert_eq!(logs.records()[1].content(), "�a");
        assert_eq!(logs.records()[2].content(), "");
        assert_eq!(logs.omitted_records(), 0);
        assert_eq!(logs.omitted_content_bytes(), 0);
        assert!(!logs.was_truncated());
    }
}
