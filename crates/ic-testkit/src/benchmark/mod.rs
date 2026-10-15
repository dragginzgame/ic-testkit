//! Parse, pair, aggregate, compare, and report compact benchmark markers.
//!
//! The normal pipeline is [`parse_benchmark_events`],
//! [`pair_benchmark_spans`], [`aggregate_benchmark_spans`], and optionally
//! [`compare_benchmark_aggregates`]. Native report output lives in
//! `benchmark::report`. Marker
//! producers can use [`format_marker`] on the host or
//! `ic_testkit::performance::Performance` in canister code.
//!
//! Totals and run counts remain exact integers. Means use IC Metrics' exact
//! scaled integer projection, rounded down. Callers choose the scale and handle
//! projection errors; reports use scale one in the original counter units.
//! Comparisons retain current and previous means instead of percentage estimates.

use std::collections::{BTreeMap, btree_map::Entry};

use ic_metrics::{MeasurementRatioError, checked_scaled_ratio};

#[cfg(not(target_arch = "wasm32"))]
pub mod report;

/// Default prefix for benchmark marker lines.
pub const DEFAULT_PREFIX: &str = "ICTK";

/// Structural identity of one suite or the cross-suite aggregate.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum BenchmarkScope {
    /// One caller-authored suite, including a suite literally named `ALL`.
    Suite(String),
    /// Aggregate across every suite.
    All,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BenchmarkParserConfig {
    pub prefixes: Vec<String>,
    pub suite_derivation: SuiteDerivation,
    /// When enabled, non-empty lines without a configured marker prefix are
    /// reported as malformed markers instead of ignored log noise.
    pub strict: bool,
}

impl Default for BenchmarkParserConfig {
    fn default() -> Self {
        Self {
            prefixes: vec![DEFAULT_PREFIX.to_string()],
            suite_derivation: SuiteDerivation::FirstPathSegment,
            strict: false,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SuiteDerivation {
    FirstPathSegment,
    Fixed(String),
}

impl SuiteDerivation {
    #[must_use]
    pub fn derive_suite(&self, span_label: &str) -> String {
        match self {
            Self::FirstPathSegment => span_label
                .split('/')
                .next()
                .filter(|part| !part.is_empty())
                .unwrap_or(span_label)
                .to_string(),
            Self::Fixed(suite) => suite.clone(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BenchmarkEventKind {
    Start,
    End,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BenchmarkEventSource {
    Unknown,
    Stdout,
    Stderr,
    FetchedLog,
}

impl BenchmarkEventSource {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Unknown => "unknown",
            Self::Stdout => "stdout",
            Self::Stderr => "stderr",
            Self::FetchedLog => "fetched_log",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct BenchmarkCounters {
    pub instructions: u128,
    pub heap_bytes: u128,
    pub memory_bytes: u128,
    pub total_allocation: u128,
}

impl BenchmarkCounters {
    fn checked_delta(self, start: Self) -> Option<Self> {
        Some(Self {
            instructions: self.instructions.checked_sub(start.instructions)?,
            heap_bytes: self.heap_bytes.checked_sub(start.heap_bytes)?,
            memory_bytes: self.memory_bytes.checked_sub(start.memory_bytes)?,
            total_allocation: self.total_allocation.checked_sub(start.total_allocation)?,
        })
    }

    fn checked_add(self, other: Self) -> Result<Self, BenchmarkCounter> {
        Ok(Self {
            instructions: self
                .instructions
                .checked_add(other.instructions)
                .ok_or(BenchmarkCounter::Instructions)?,
            heap_bytes: self
                .heap_bytes
                .checked_add(other.heap_bytes)
                .ok_or(BenchmarkCounter::HeapBytes)?,
            memory_bytes: self
                .memory_bytes
                .checked_add(other.memory_bytes)
                .ok_or(BenchmarkCounter::MemoryBytes)?,
            total_allocation: self
                .total_allocation
                .checked_add(other.total_allocation)
                .ok_or(BenchmarkCounter::TotalAllocation)?,
        })
    }

    fn min_assign(&mut self, other: Self) {
        self.instructions = self.instructions.min(other.instructions);
        self.heap_bytes = self.heap_bytes.min(other.heap_bytes);
        self.memory_bytes = self.memory_bytes.min(other.memory_bytes);
        self.total_allocation = self.total_allocation.min(other.total_allocation);
    }

    fn max_assign(&mut self, other: Self) {
        self.instructions = self.instructions.max(other.instructions);
        self.heap_bytes = self.heap_bytes.max(other.heap_bytes);
        self.memory_bytes = self.memory_bytes.max(other.memory_bytes);
        self.total_allocation = self.total_allocation.max(other.total_allocation);
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RawBenchmarkEvent {
    pub prefix: String,
    pub label: String,
    pub suite: String,
    pub span_label: String,
    pub kind: BenchmarkEventKind,
    pub counters: BenchmarkCounters,
    pub source_line: usize,
    pub source: BenchmarkEventSource,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MalformedBenchmarkMarker {
    pub source_line: usize,
    pub source: BenchmarkEventSource,
    pub line: String,
    pub reason: String,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct BenchmarkParseReport {
    pub events: Vec<RawBenchmarkEvent>,
    pub malformed_markers: Vec<MalformedBenchmarkMarker>,
    pub ignored_line_count: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BenchmarkSpan {
    pub suite: String,
    pub span_label: String,
    pub start_line: usize,
    pub end_line: usize,
    pub start: BenchmarkCounters,
    pub end: BenchmarkCounters,
    pub delta: BenchmarkCounters,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UnpairedBenchmarkMarkerKind {
    Start,
    End,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UnpairedBenchmarkMarker {
    pub event: RawBenchmarkEvent,
    pub kind: UnpairedBenchmarkMarkerKind,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InvalidBenchmarkSpan {
    pub start: RawBenchmarkEvent,
    pub end: RawBenchmarkEvent,
    pub reason: String,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct BenchmarkSpanReport {
    pub spans: Vec<BenchmarkSpan>,
    pub unpaired_markers: Vec<UnpairedBenchmarkMarker>,
    pub invalid_spans: Vec<InvalidBenchmarkSpan>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BenchmarkAggregateRow {
    pub span_label: String,
    pub runs: u64,
    pub total: BenchmarkCounters,
    pub min: BenchmarkCounters,
    pub max: BenchmarkCounters,
    pub peak_end: BenchmarkCounters,
    /// Suite identity or the cross-suite aggregate.
    pub scope: BenchmarkScope,
}

/// Floor-rounded scaled means of exact aggregate totals and run counts.
///
/// Fields are `floor(total * scale / runs)`; choose a positive scale to
/// interpret them in the original counter unit. Scale zero projects zero.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BenchmarkAverages {
    pub scale: u64,
    pub instructions: u128,
    pub heap_bytes: u128,
    pub memory_bytes: u128,
    pub total_allocation: u128,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct BenchmarkAggregateReport {
    pub rows: Vec<BenchmarkAggregateRow>,
}

/// Counter whose aggregate cannot be represented.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BenchmarkCounter {
    Runs,
    Instructions,
    HeapBytes,
    MemoryBytes,
    TotalAllocation,
}

/// Overflow rejects the complete aggregation, without returning partial totals.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BenchmarkAggregateError {
    pub span_label: String,
    pub counter: BenchmarkCounter,
    /// Suite identity or the cross-suite aggregate.
    pub scope: BenchmarkScope,
}

impl std::fmt::Display for BenchmarkAggregateError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "benchmark aggregate overflow: scope={:?}, span={}, counter={:?}",
            self.scope, self.span_label, self.counter
        )
    }
}

impl std::error::Error for BenchmarkAggregateError {}

/// Current and previous scaled means, preserving missing rows and measured zero.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BenchmarkComparisonRow {
    pub span_label: String,
    pub current_runs: Option<u64>,
    pub previous_runs: Option<u64>,
    pub current_average: Option<BenchmarkAverages>,
    pub previous_average: Option<BenchmarkAverages>,
    /// Suite identity or the cross-suite aggregate.
    pub scope: BenchmarkScope,
}

impl BenchmarkAggregateRow {
    /// Project exact totals at the caller's scale, rounding down.
    /// Zero runs and unrepresentable scaled results return IC Metrics' typed error.
    pub fn average(&self, scale: u64) -> Result<BenchmarkAverages, MeasurementRatioError> {
        Ok(BenchmarkAverages {
            scale,
            instructions: checked_scaled_ratio(self.total.instructions, self.runs, scale)?,
            heap_bytes: checked_scaled_ratio(self.total.heap_bytes, self.runs, scale)?,
            memory_bytes: checked_scaled_ratio(self.total.memory_bytes, self.runs, scale)?,
            total_allocation: checked_scaled_ratio(self.total.total_allocation, self.runs, scale)?,
        })
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct BenchmarkComparisonReport {
    pub rows: Vec<BenchmarkComparisonRow>,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct BenchmarkRunMetadata {
    pub timestamp: String,
    pub run_directory_name: String,
    pub run_index: u32,
    pub git_commit_hash: Option<String>,
    pub git_commit_short_hash: Option<String>,
    pub ic_testkit_version: String,
    pub pocket_ic_version: String,
    pub rustc_version: String,
    pub benchmark_command: Option<String>,
    pub selected_previous_run: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BenchmarkRunReport {
    pub parse: BenchmarkParseReport,
    pub spans: BenchmarkSpanReport,
    pub aggregates: BenchmarkAggregateReport,
    pub comparison: Option<BenchmarkComparisonReport>,
    pub metadata: BenchmarkRunMetadata,
}

#[must_use]
pub fn format_marker(prefix: &str, label: &str, counters: BenchmarkCounters) -> String {
    format!(
        "{}|{}|{}|{}|{}|{}",
        prefix,
        label,
        counters.instructions,
        counters.heap_bytes,
        counters.memory_bytes,
        counters.total_allocation
    )
}

#[must_use]
pub fn parse_benchmark_events(input: &str, config: &BenchmarkParserConfig) -> BenchmarkParseReport {
    parse_benchmark_events_from_source(input, config, BenchmarkEventSource::Unknown)
}

#[must_use]
pub fn parse_benchmark_events_from_source(
    input: &str,
    config: &BenchmarkParserConfig,
    source: BenchmarkEventSource,
) -> BenchmarkParseReport {
    let mut report = BenchmarkParseReport::default();
    append_benchmark_events(&mut report, input, config, source);
    report
}

fn append_benchmark_events(
    report: &mut BenchmarkParseReport,
    input: &str,
    config: &BenchmarkParserConfig,
    source: BenchmarkEventSource,
) {
    for (index, line) in input.lines().enumerate() {
        let source_line = index + 1;
        if !has_configured_prefix(line, &config.prefixes) {
            if config.strict && !line.trim().is_empty() {
                report.malformed_markers.push(malformed(
                    source_line,
                    source,
                    line,
                    "line does not use a configured marker prefix",
                ));
            } else {
                report.ignored_line_count += 1;
            }
            continue;
        }

        match parse_marker_line(line, source_line, source, config) {
            Ok(event) => report.events.push(event),
            Err(marker) => report.malformed_markers.push(marker),
        }
    }
}

/// Parse separately captured stdout and stderr.
///
/// Separate streams do not carry global ordering, so events are returned in
/// stdout-then-stderr order. If a benchmark span can start on one stream and end
/// on the other, capture combined process output and use [`parse_benchmark_events`].
#[must_use]
pub fn parse_benchmark_events_from_captured_output(
    stdout: &str,
    stderr: &str,
    config: &BenchmarkParserConfig,
) -> BenchmarkParseReport {
    let mut report = BenchmarkParseReport::default();
    append_benchmark_events(&mut report, stdout, config, BenchmarkEventSource::Stdout);
    append_benchmark_events(&mut report, stderr, config, BenchmarkEventSource::Stderr);
    report
}

#[must_use]
pub fn pair_benchmark_spans(events: &[RawBenchmarkEvent]) -> BenchmarkSpanReport {
    let mut report = BenchmarkSpanReport::default();
    let mut open_starts: BTreeMap<(&str, &str), Vec<&RawBenchmarkEvent>> = BTreeMap::new();

    for event in events {
        let key = (event.suite.as_str(), event.span_label.as_str());
        match event.kind {
            BenchmarkEventKind::Start => open_starts.entry(key).or_default().push(event),
            BenchmarkEventKind::End => match open_starts.entry(key) {
                Entry::Occupied(mut entry) => {
                    let start = entry
                        .get_mut()
                        .pop()
                        .expect("open marker stacks must remain nonempty");
                    if entry.get().is_empty() {
                        entry.remove();
                    }
                    push_paired_span(&mut report, start, event);
                }
                Entry::Vacant(_) => report.unpaired_markers.push(UnpairedBenchmarkMarker {
                    event: event.clone(),
                    kind: UnpairedBenchmarkMarkerKind::End,
                }),
            },
        }
    }

    for starts in open_starts.into_values() {
        for event in starts {
            report.unpaired_markers.push(UnpairedBenchmarkMarker {
                event: event.clone(),
                kind: UnpairedBenchmarkMarkerKind::Start,
            });
        }
    }

    report
}

/// Aggregate spans by named suite and across all suites.
///
/// Returns an error if any counter total or run count overflows. No partial
/// report is returned, and totals are never wrapped or saturated.
pub fn aggregate_benchmark_spans(
    spans: &[BenchmarkSpan],
) -> Result<BenchmarkAggregateReport, BenchmarkAggregateError> {
    let mut rows: BTreeMap<(BenchmarkScope, &str), AggregateBuilder> = BTreeMap::new();

    for span in spans {
        add_span_to_aggregate(&mut rows, BenchmarkScope::Suite(span.suite.clone()), span)?;
        add_span_to_aggregate(&mut rows, BenchmarkScope::All, span)?;
    }

    Ok(BenchmarkAggregateReport {
        rows: rows
            .into_iter()
            .map(|((scope, span_label), builder)| builder.finish(scope, span_label.to_owned()))
            .collect(),
    })
}

/// Pair current and previous means at the same caller-selected scale.
/// Returns a typed error if either side cannot supply the selected projection.
pub fn compare_benchmark_aggregates(
    current: &[BenchmarkAggregateRow],
    previous: &[BenchmarkAggregateRow],
    scale: u64,
) -> Result<BenchmarkComparisonReport, MeasurementRatioError> {
    let mut rows = BTreeMap::new();
    for row in current {
        rows.insert((&row.scope, row.span_label.as_str()), (Some(row), None));
    }
    for row in previous {
        rows.entry((&row.scope, row.span_label.as_str()))
            .or_insert((None, None))
            .1 = Some(row);
    }

    Ok(BenchmarkComparisonReport {
        rows: rows
            .into_iter()
            .map(|((scope, span_label), (current_row, previous_row))| {
                Ok(BenchmarkComparisonRow {
                    span_label: span_label.to_owned(),
                    current_runs: current_row.map(|row| row.runs),
                    previous_runs: previous_row.map(|row| row.runs),
                    current_average: current_row.map(|row| row.average(scale)).transpose()?,
                    previous_average: previous_row.map(|row| row.average(scale)).transpose()?,
                    scope: scope.clone(),
                })
            })
            .collect::<Result<_, MeasurementRatioError>>()?,
    })
}

fn parse_marker_line(
    line: &str,
    source_line: usize,
    source: BenchmarkEventSource,
    config: &BenchmarkParserConfig,
) -> Result<RawBenchmarkEvent, MalformedBenchmarkMarker> {
    let mut parts = line.split('|');
    // Six fields and an absent seventh establish the complete marker shape
    // without collecting an arbitrary number of columns from malformed logs.
    let columns: [Option<&str>; 7] = std::array::from_fn(|_| parts.next());
    let [
        Some(prefix),
        Some(label),
        Some(instructions),
        Some(heap_bytes),
        Some(memory_bytes),
        Some(total_allocation),
        None,
    ] = columns
    else {
        return Err(malformed(
            source_line,
            source,
            line,
            "expected six pipe-separated columns",
        ));
    };

    if !config.prefixes.iter().any(|known| known == prefix) {
        return Err(malformed(
            source_line,
            source,
            line,
            "prefix is not configured",
        ));
    }

    if label.is_empty() {
        return Err(malformed(source_line, source, line, "label is empty"));
    }

    let (span_label, kind) = split_label_kind(label).ok_or_else(|| {
        malformed(
            source_line,
            source,
            line,
            "label must end in :start or :end",
        )
    })?;

    let counters = BenchmarkCounters {
        instructions: parse_counter(instructions, source_line, source, line, "instructions")?,
        heap_bytes: parse_counter(heap_bytes, source_line, source, line, "heap_bytes")?,
        memory_bytes: parse_counter(memory_bytes, source_line, source, line, "memory_bytes")?,
        total_allocation: parse_counter(
            total_allocation,
            source_line,
            source,
            line,
            "total_allocation",
        )?,
    };
    let suite = config.suite_derivation.derive_suite(span_label);

    Ok(RawBenchmarkEvent {
        prefix: prefix.to_string(),
        label: label.to_string(),
        suite,
        span_label: span_label.to_string(),
        kind,
        counters,
        source_line,
        source,
    })
}

fn parse_counter(
    value: &str,
    source_line: usize,
    source: BenchmarkEventSource,
    line: &str,
    name: &str,
) -> Result<u128, MalformedBenchmarkMarker> {
    if value.is_empty() {
        return Err(malformed(
            source_line,
            source,
            line,
            &format!("{name} counter is empty"),
        ));
    }

    value.parse::<u128>().map_err(|_| {
        malformed(
            source_line,
            source,
            line,
            &format!("{name} counter is not an unsigned integer"),
        )
    })
}

fn split_label_kind(label: &str) -> Option<(&str, BenchmarkEventKind)> {
    let start = label.strip_suffix(":start");
    let end = label.strip_suffix(":end");

    match (start, end) {
        (Some(span_label), None) if !span_label.is_empty() => {
            Some((span_label, BenchmarkEventKind::Start))
        }
        (None, Some(span_label)) if !span_label.is_empty() => {
            Some((span_label, BenchmarkEventKind::End))
        }
        _ => None,
    }
}

fn has_configured_prefix(line: &str, prefixes: &[String]) -> bool {
    prefixes.iter().any(|prefix| {
        line.strip_prefix(prefix)
            .is_some_and(|rest| rest.starts_with('|'))
    })
}

fn malformed(
    source_line: usize,
    source: BenchmarkEventSource,
    line: &str,
    reason: &str,
) -> MalformedBenchmarkMarker {
    MalformedBenchmarkMarker {
        source_line,
        source,
        line: line.to_string(),
        reason: reason.to_string(),
    }
}

fn push_paired_span(
    report: &mut BenchmarkSpanReport,
    start: &RawBenchmarkEvent,
    end: &RawBenchmarkEvent,
) {
    if let Some(delta) = end.counters.checked_delta(start.counters) {
        report.spans.push(BenchmarkSpan {
            suite: start.suite.clone(),
            span_label: start.span_label.clone(),
            start_line: start.source_line,
            end_line: end.source_line,
            start: start.counters,
            end: end.counters,
            delta,
        });
    } else {
        report.invalid_spans.push(InvalidBenchmarkSpan {
            start: start.clone(),
            end: end.clone(),
            reason: "end counter is lower than start counter".to_string(),
        });
    }
}

#[derive(Clone, Debug)]
struct AggregateBuilder {
    runs: u64,
    total: BenchmarkCounters,
    min: BenchmarkCounters,
    max: BenchmarkCounters,
    peak_end: BenchmarkCounters,
}

impl AggregateBuilder {
    const fn new(span: &BenchmarkSpan) -> Self {
        Self {
            runs: 1,
            total: span.delta,
            min: span.delta,
            max: span.delta,
            peak_end: span.end,
        }
    }

    fn push(&mut self, span: &BenchmarkSpan) -> Result<(), BenchmarkCounter> {
        let runs = self.runs.checked_add(1).ok_or(BenchmarkCounter::Runs)?;
        let total = self.total.checked_add(span.delta)?;
        self.runs = runs;
        self.total = total;
        self.min.min_assign(span.delta);
        self.max.max_assign(span.delta);
        self.peak_end.max_assign(span.end);
        Ok(())
    }

    const fn finish(self, scope: BenchmarkScope, span_label: String) -> BenchmarkAggregateRow {
        BenchmarkAggregateRow {
            span_label,
            runs: self.runs,
            total: self.total,
            min: self.min,
            max: self.max,
            peak_end: self.peak_end,
            scope,
        }
    }
}

fn add_span_to_aggregate<'a>(
    rows: &mut BTreeMap<(BenchmarkScope, &'a str), AggregateBuilder>,
    scope: BenchmarkScope,
    span: &'a BenchmarkSpan,
) -> Result<(), BenchmarkAggregateError> {
    match rows.entry((scope, span.span_label.as_str())) {
        Entry::Occupied(mut entry) => entry.get_mut().push(span).map_err(|counter| {
            let (scope, span_label) = entry.key();
            BenchmarkAggregateError {
                span_label: (*span_label).to_owned(),
                counter,
                scope: scope.clone(),
            }
        }),
        Entry::Vacant(entry) => {
            entry.insert(AggregateBuilder::new(span));
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{AggregateBuilder, BenchmarkCounter, BenchmarkCounters, BenchmarkSpan};

    #[test]
    fn aggregate_run_count_overflow_is_rejected_without_changing_totals() {
        let span = BenchmarkSpan {
            suite: "probe".into(),
            span_label: "probe/run".into(),
            start_line: 1,
            end_line: 2,
            start: BenchmarkCounters::default(),
            end: BenchmarkCounters::default(),
            delta: BenchmarkCounters::default(),
        };
        let mut builder = AggregateBuilder::new(&span);
        builder.runs = u64::MAX;
        assert_eq!(builder.push(&span), Err(BenchmarkCounter::Runs));
        assert_eq!(builder.runs, u64::MAX);
        assert_eq!(builder.total, BenchmarkCounters::default());
    }
}
