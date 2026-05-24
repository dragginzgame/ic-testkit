//! Log-marker benchmarking helpers for PocketIC-style test harnesses.

use std::{
    collections::{BTreeMap, btree_map::Entry},
    ffi::OsStr,
    fmt::Write as _,
    fs, io,
    path::{Path, PathBuf},
};

use serde_json::Value;

pub const DEFAULT_PREFIX: &str = "ICTK";
pub const ALL_SUITES: &str = "ALL";

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

    const fn add_assign(&mut self, other: Self) {
        self.instructions += other.instructions;
        self.heap_bytes += other.heap_bytes;
        self.memory_bytes += other.memory_bytes;
        self.total_allocation += other.total_allocation;
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

#[derive(Clone, Debug, PartialEq)]
pub struct BenchmarkAggregateRow {
    pub suite: String,
    pub span_label: String,
    pub runs: u64,
    pub total: BenchmarkCounters,
    pub average: BenchmarkAverages,
    pub min: BenchmarkCounters,
    pub max: BenchmarkCounters,
    pub peak_end: BenchmarkCounters,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BenchmarkAverages {
    pub instructions: f64,
    pub heap_bytes: f64,
    pub memory_bytes: f64,
    pub total_allocation: f64,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct BenchmarkAggregateReport {
    pub rows: Vec<BenchmarkAggregateRow>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BenchmarkComparisonRow {
    pub suite: String,
    pub span_label: String,
    pub current_runs: Option<u64>,
    pub previous_runs: Option<u64>,
    pub instructions_avg_change_percent: Option<f64>,
    pub heap_bytes_avg_change_percent: Option<f64>,
    pub memory_bytes_avg_change_percent: Option<f64>,
    pub total_allocation_avg_change_percent: Option<f64>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct BenchmarkComparisonReport {
    pub rows: Vec<BenchmarkComparisonRow>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
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

#[derive(Clone, Debug, PartialEq)]
pub struct BenchmarkRunReport {
    pub parse: BenchmarkParseReport,
    pub spans: BenchmarkSpanReport,
    pub aggregates: BenchmarkAggregateReport,
    pub comparison: Option<BenchmarkComparisonReport>,
    pub metadata: BenchmarkRunMetadata,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BenchmarkRunDirectory {
    pub path: PathBuf,
    pub directory_name: String,
    pub run_index: u32,
    pub git_commit_hash: Option<String>,
    pub git_commit_short_hash: Option<String>,
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
pub fn benchmark_run_directory_name(
    timestamp: &str,
    git_commit_short_hash: Option<&str>,
    run_index: u32,
) -> String {
    let commit = git_commit_short_hash
        .filter(|hash| !hash.is_empty())
        .unwrap_or("unknown");
    format!("{timestamp}-{commit}-{run_index:04}")
}

pub fn next_benchmark_run_directory(
    runs_root: impl AsRef<Path>,
    timestamp: &str,
    git_commit_hash: Option<&str>,
) -> io::Result<BenchmarkRunDirectory> {
    let runs_root = runs_root.as_ref();
    let git_commit_short_hash = git_commit_hash.map(short_commit_hash);
    let prefix = format!(
        "{}-{}-",
        timestamp,
        git_commit_short_hash.as_deref().unwrap_or("unknown")
    );
    let run_index = next_run_index_for_prefix(runs_root, &prefix)?;
    let directory_name =
        benchmark_run_directory_name(timestamp, git_commit_short_hash.as_deref(), run_index);

    Ok(BenchmarkRunDirectory {
        path: runs_root.join(&directory_name),
        directory_name,
        run_index,
        git_commit_hash: git_commit_hash.map(str::to_string),
        git_commit_short_hash,
    })
}

pub fn find_latest_previous_run(
    runs_root: impl AsRef<Path>,
    current_run_directory_name: &str,
    benchmark_command: Option<&str>,
) -> io::Result<Option<PathBuf>> {
    let runs_root = runs_root.as_ref();
    let mut candidates = Vec::new();

    if !runs_root.exists() {
        return Ok(None);
    }

    for entry in fs::read_dir(runs_root)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }

        let directory_name = entry.file_name().to_string_lossy().into_owned();
        if directory_name == current_run_directory_name
            || directory_name.as_str() > current_run_directory_name
        {
            continue;
        }

        let metadata_path = entry.path().join("metadata.json");
        let Ok(metadata) = read_benchmark_run_metadata(&metadata_path) else {
            continue;
        };

        if let Some(command) = benchmark_command
            && metadata.benchmark_command.as_deref() != Some(command)
        {
            continue;
        }

        candidates.push((metadata.timestamp, directory_name, entry.path()));
    }

    candidates.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));
    Ok(candidates.pop().map(|(_, _, path)| path))
}

pub fn read_benchmark_run_metadata(path: impl AsRef<Path>) -> io::Result<BenchmarkRunMetadata> {
    let input = fs::read_to_string(path)?;
    let value = serde_json::from_str::<Value>(&input).map_err(metadata_json_error)?;

    Ok(BenchmarkRunMetadata {
        timestamp: metadata_required_string(&value, "timestamp")?,
        run_directory_name: metadata_required_string(&value, "run_directory_name")?,
        run_index: metadata_required_u32(&value, "run_index")?,
        git_commit_hash: metadata_optional_string(&value, "git_commit_hash")?,
        git_commit_short_hash: metadata_optional_string(&value, "git_commit_short_hash")?,
        ic_testkit_version: metadata_required_string(&value, "ic_testkit_version")?,
        pocket_ic_version: metadata_required_string(&value, "pocket_ic_version")?,
        rustc_version: metadata_required_string(&value, "rustc_version")?,
        benchmark_command: metadata_optional_string(&value, "benchmark_command")?,
        selected_previous_run: metadata_optional_string(&value, "selected_previous_run")?,
    })
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

    report
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
    let mut report =
        parse_benchmark_events_from_source(stdout, config, BenchmarkEventSource::Stdout);
    let stderr_report =
        parse_benchmark_events_from_source(stderr, config, BenchmarkEventSource::Stderr);

    report.events.extend(stderr_report.events);
    report
        .malformed_markers
        .extend(stderr_report.malformed_markers);
    report.ignored_line_count += stderr_report.ignored_line_count;
    report
}

#[must_use]
pub fn pair_benchmark_spans(events: &[RawBenchmarkEvent]) -> BenchmarkSpanReport {
    let mut report = BenchmarkSpanReport::default();
    let mut open_starts: BTreeMap<(String, String), Vec<RawBenchmarkEvent>> = BTreeMap::new();

    for event in events {
        let key = (event.suite.clone(), event.span_label.clone());
        match event.kind {
            BenchmarkEventKind::Start => open_starts.entry(key).or_default().push(event.clone()),
            BenchmarkEventKind::End => match open_starts.entry(key) {
                Entry::Occupied(mut entry) => {
                    if let Some(start) = entry.get_mut().pop() {
                        if entry.get().is_empty() {
                            entry.remove();
                        }
                        push_paired_span(&mut report, start, event.clone());
                    } else {
                        report.unpaired_markers.push(UnpairedBenchmarkMarker {
                            event: event.clone(),
                            kind: UnpairedBenchmarkMarkerKind::End,
                        });
                    }
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
                event,
                kind: UnpairedBenchmarkMarkerKind::Start,
            });
        }
    }

    report
}

#[must_use]
pub fn aggregate_benchmark_spans(spans: &[BenchmarkSpan]) -> BenchmarkAggregateReport {
    let mut rows: BTreeMap<(String, String), AggregateBuilder> = BTreeMap::new();

    for span in spans {
        add_span_to_aggregate(&mut rows, &span.suite, &span.span_label, span);
        add_span_to_aggregate(&mut rows, ALL_SUITES, &span.span_label, span);
    }

    BenchmarkAggregateReport {
        rows: rows.into_values().map(AggregateBuilder::finish).collect(),
    }
}

#[must_use]
pub fn compare_benchmark_aggregates(
    current: &[BenchmarkAggregateRow],
    previous: &[BenchmarkAggregateRow],
) -> BenchmarkComparisonReport {
    let current_by_key = aggregate_rows_by_key(current);
    let previous_by_key = aggregate_rows_by_key(previous);
    let mut keys = current_by_key.keys().cloned().collect::<Vec<_>>();

    for key in previous_by_key.keys() {
        if !current_by_key.contains_key(key) {
            keys.push(key.clone());
        }
    }

    keys.sort();
    keys.dedup();

    BenchmarkComparisonReport {
        rows: keys
            .into_iter()
            .map(|(suite, span_label)| {
                let current_row = current_by_key.get(&(suite.clone(), span_label.clone()));
                let previous_row = previous_by_key.get(&(suite.clone(), span_label.clone()));
                BenchmarkComparisonRow {
                    suite,
                    span_label,
                    current_runs: current_row.map(|row| row.runs),
                    previous_runs: previous_row.map(|row| row.runs),
                    instructions_avg_change_percent: compare_average(
                        current_row.map(|row| row.average.instructions),
                        previous_row.map(|row| row.average.instructions),
                    ),
                    heap_bytes_avg_change_percent: compare_average(
                        current_row.map(|row| row.average.heap_bytes),
                        previous_row.map(|row| row.average.heap_bytes),
                    ),
                    memory_bytes_avg_change_percent: compare_average(
                        current_row.map(|row| row.average.memory_bytes),
                        previous_row.map(|row| row.average.memory_bytes),
                    ),
                    total_allocation_avg_change_percent: compare_average(
                        current_row.map(|row| row.average.total_allocation),
                        previous_row.map(|row| row.average.total_allocation),
                    ),
                }
            })
            .collect(),
    }
}

pub fn write_benchmark_report_dir(
    report: &BenchmarkRunReport,
    path: impl AsRef<Path>,
) -> io::Result<()> {
    let path = path.as_ref();
    fs::create_dir_all(path)?;

    fs::write(
        path.join("raw-events.csv"),
        raw_events_csv(&report.parse.events),
    )?;
    fs::write(
        path.join("malformed-markers.csv"),
        malformed_markers_csv(&report.parse.malformed_markers),
    )?;
    fs::write(path.join("spans.csv"), spans_csv(&report.spans.spans))?;
    fs::write(
        path.join("unpaired-markers.csv"),
        unpaired_markers_csv(&report.spans.unpaired_markers),
    )?;
    fs::write(
        path.join("invalid-spans.csv"),
        invalid_spans_csv(&report.spans.invalid_spans),
    )?;
    fs::write(
        path.join("suite-aggregates.csv"),
        aggregates_csv(
            report
                .aggregates
                .rows
                .iter()
                .filter(|row| row.suite != ALL_SUITES),
        ),
    )?;
    fs::write(
        path.join("all-aggregates.csv"),
        aggregates_csv(
            report
                .aggregates
                .rows
                .iter()
                .filter(|row| row.suite == ALL_SUITES),
        ),
    )?;
    fs::write(
        path.join("comparison.csv"),
        comparison_csv(report.comparison.as_ref()),
    )?;
    fs::write(
        path.join("bench-summary.md"),
        benchmark_summary_markdown(report),
    )?;
    fs::write(path.join("metadata.json"), metadata_json(&report.metadata))?;

    Ok(())
}

fn parse_marker_line(
    line: &str,
    source_line: usize,
    source: BenchmarkEventSource,
    config: &BenchmarkParserConfig,
) -> Result<RawBenchmarkEvent, MalformedBenchmarkMarker> {
    let parts = line.split('|').collect::<Vec<_>>();
    if parts.len() != 6 {
        return Err(malformed(
            source_line,
            source,
            line,
            "expected six pipe-separated columns",
        ));
    }

    let prefix = parts[0];
    if !config.prefixes.iter().any(|known| known == prefix) {
        return Err(malformed(
            source_line,
            source,
            line,
            "prefix is not configured",
        ));
    }

    let label = parts[1];
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
        instructions: parse_counter(parts[2], source_line, source, line, "instructions")?,
        heap_bytes: parse_counter(parts[3], source_line, source, line, "heap_bytes")?,
        memory_bytes: parse_counter(parts[4], source_line, source, line, "memory_bytes")?,
        total_allocation: parse_counter(parts[5], source_line, source, line, "total_allocation")?,
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
    start: RawBenchmarkEvent,
    end: RawBenchmarkEvent,
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
            start,
            end,
            reason: "end counter is lower than start counter".to_string(),
        });
    }
}

#[derive(Clone, Debug)]
struct AggregateBuilder {
    suite: String,
    span_label: String,
    runs: u64,
    total: BenchmarkCounters,
    min: BenchmarkCounters,
    max: BenchmarkCounters,
    peak_end: BenchmarkCounters,
}

impl AggregateBuilder {
    fn new(suite: &str, span_label: &str, span: &BenchmarkSpan) -> Self {
        Self {
            suite: suite.to_string(),
            span_label: span_label.to_string(),
            runs: 1,
            total: span.delta,
            min: span.delta,
            max: span.delta,
            peak_end: span.end,
        }
    }

    fn push(&mut self, span: &BenchmarkSpan) {
        self.runs += 1;
        self.total.add_assign(span.delta);
        self.min.min_assign(span.delta);
        self.max.max_assign(span.delta);
        self.peak_end.max_assign(span.end);
    }

    fn finish(self) -> BenchmarkAggregateRow {
        BenchmarkAggregateRow {
            suite: self.suite,
            span_label: self.span_label,
            runs: self.runs,
            total: self.total,
            average: averages(self.total, self.runs),
            min: self.min,
            max: self.max,
            peak_end: self.peak_end,
        }
    }
}

fn add_span_to_aggregate(
    rows: &mut BTreeMap<(String, String), AggregateBuilder>,
    suite: &str,
    span_label: &str,
    span: &BenchmarkSpan,
) {
    match rows.entry((suite.to_string(), span_label.to_string())) {
        Entry::Occupied(mut entry) => entry.get_mut().push(span),
        Entry::Vacant(entry) => {
            entry.insert(AggregateBuilder::new(suite, span_label, span));
        }
    }
}

#[expect(clippy::cast_precision_loss)]
fn averages(total: BenchmarkCounters, runs: u64) -> BenchmarkAverages {
    let runs = runs as f64;
    BenchmarkAverages {
        instructions: total.instructions as f64 / runs,
        heap_bytes: total.heap_bytes as f64 / runs,
        memory_bytes: total.memory_bytes as f64 / runs,
        total_allocation: total.total_allocation as f64 / runs,
    }
}

fn aggregate_rows_by_key(
    rows: &[BenchmarkAggregateRow],
) -> BTreeMap<(String, String), &BenchmarkAggregateRow> {
    rows.iter()
        .map(|row| ((row.suite.clone(), row.span_label.clone()), row))
        .collect()
}

fn compare_average(current: Option<f64>, previous: Option<f64>) -> Option<f64> {
    match (current, previous) {
        (Some(current), Some(previous)) if previous != 0.0 => {
            Some(((current - previous) / previous) * 100.0)
        }
        _ => None,
    }
}

fn raw_events_csv(events: &[RawBenchmarkEvent]) -> String {
    let mut out = String::from(
        "source_line,source,prefix,suite,label,span_label,kind,instructions,heap_bytes,memory_bytes,total_allocation\n",
    );
    for event in events {
        let _ = writeln!(
            out,
            "{},{},{},{},{},{},{},{},{},{},{}",
            event.source_line,
            event.source.as_str(),
            csv_cell(&event.prefix),
            csv_cell(&event.suite),
            csv_cell(&event.label),
            csv_cell(&event.span_label),
            kind_str(event.kind),
            event.counters.instructions,
            event.counters.heap_bytes,
            event.counters.memory_bytes,
            event.counters.total_allocation
        );
    }
    out
}

fn malformed_markers_csv(markers: &[MalformedBenchmarkMarker]) -> String {
    let mut out = String::from("source_line,source,reason,line\n");
    for marker in markers {
        let _ = writeln!(
            out,
            "{},{},{},{}",
            marker.source_line,
            marker.source.as_str(),
            csv_cell(&marker.reason),
            csv_cell(&marker.line)
        );
    }
    out
}

fn spans_csv(spans: &[BenchmarkSpan]) -> String {
    let mut out = String::from(
        "suite,span_label,start_line,end_line,instructions_delta,heap_bytes_delta,memory_bytes_delta,total_allocation_delta\n",
    );
    for span in spans {
        let _ = writeln!(
            out,
            "{},{},{},{},{},{},{},{}",
            csv_cell(&span.suite),
            csv_cell(&span.span_label),
            span.start_line,
            span.end_line,
            span.delta.instructions,
            span.delta.heap_bytes,
            span.delta.memory_bytes,
            span.delta.total_allocation
        );
    }
    out
}

fn unpaired_markers_csv(markers: &[UnpairedBenchmarkMarker]) -> String {
    let mut out = String::from("source_line,source,kind,suite,span_label,label\n");
    for marker in markers {
        let kind = match marker.kind {
            UnpairedBenchmarkMarkerKind::Start => "start",
            UnpairedBenchmarkMarkerKind::End => "end",
        };
        let _ = writeln!(
            out,
            "{},{},{},{},{},{}",
            marker.event.source_line,
            marker.event.source.as_str(),
            kind,
            csv_cell(&marker.event.suite),
            csv_cell(&marker.event.span_label),
            csv_cell(&marker.event.label)
        );
    }
    out
}

fn invalid_spans_csv(spans: &[InvalidBenchmarkSpan]) -> String {
    let mut out = String::from("start_line,end_line,suite,span_label,reason\n");
    for span in spans {
        let _ = writeln!(
            out,
            "{},{},{},{},{}",
            span.start.source_line,
            span.end.source_line,
            csv_cell(&span.start.suite),
            csv_cell(&span.start.span_label),
            csv_cell(&span.reason)
        );
    }
    out
}

fn aggregates_csv<'a>(rows: impl Iterator<Item = &'a BenchmarkAggregateRow>) -> String {
    let mut out = String::from(
        "suite,span_label,runs,instructions_total,instructions_avg,heap_bytes_total,heap_bytes_avg,memory_bytes_total,memory_bytes_avg,total_allocation_total,total_allocation_avg\n",
    );
    for row in rows {
        let _ = writeln!(
            out,
            "{},{},{},{},{:.4},{},{:.4},{},{:.4},{},{:.4}",
            csv_cell(&row.suite),
            csv_cell(&row.span_label),
            row.runs,
            row.total.instructions,
            row.average.instructions,
            row.total.heap_bytes,
            row.average.heap_bytes,
            row.total.memory_bytes,
            row.average.memory_bytes,
            row.total.total_allocation,
            row.average.total_allocation
        );
    }
    out
}

fn comparison_csv(comparison: Option<&BenchmarkComparisonReport>) -> String {
    let mut out = String::from(
        "suite,span_label,current_runs,previous_runs,instructions_avg_change_percent,heap_bytes_avg_change_percent,memory_bytes_avg_change_percent,total_allocation_avg_change_percent\n",
    );

    let Some(comparison) = comparison else {
        return out;
    };

    for row in &comparison.rows {
        let _ = writeln!(
            out,
            "{},{},{},{},{},{},{},{}",
            csv_cell(&row.suite),
            csv_cell(&row.span_label),
            optional_u64_cell(row.current_runs),
            optional_u64_cell(row.previous_runs),
            optional_f64_cell(row.instructions_avg_change_percent),
            optional_f64_cell(row.heap_bytes_avg_change_percent),
            optional_f64_cell(row.memory_bytes_avg_change_percent),
            optional_f64_cell(row.total_allocation_avg_change_percent)
        );
    }

    out
}

fn benchmark_summary_markdown(report: &BenchmarkRunReport) -> String {
    let comparison_by_key = report.comparison.as_ref().map(|comparison| {
        comparison
            .rows
            .iter()
            .map(|row| ((row.suite.clone(), row.span_label.clone()), row))
            .collect::<BTreeMap<_, _>>()
    });
    let mut out = String::from(
        "# Benchmark Summary\n\n| Benchmark | Runs | Instructions Avg | Heap Delta Avg | Memory Delta Avg | Allocation Avg |\n| --- | ---: | ---: | ---: | ---: | ---: |\n",
    );

    for row in report
        .aggregates
        .rows
        .iter()
        .filter(|row| row.suite != ALL_SUITES)
    {
        let comparison = comparison_by_key.as_ref().and_then(|rows| {
            rows.get(&(row.suite.clone(), row.span_label.clone()))
                .copied()
        });
        let _ = writeln!(
            out,
            "| {} | {} | {} | {} | {} | {} |",
            markdown_cell(&row.span_label),
            row.runs,
            format_instructions(
                row.average.instructions,
                change_suffix(comparison, |c| { c.instructions_avg_change_percent })
            ),
            format_bytes(
                row.average.heap_bytes,
                change_suffix(comparison, |c| c.heap_bytes_avg_change_percent)
            ),
            format_bytes(
                row.average.memory_bytes,
                change_suffix(comparison, |c| c.memory_bytes_avg_change_percent)
            ),
            format_bytes(
                row.average.total_allocation,
                change_suffix(comparison, |c| c.total_allocation_avg_change_percent)
            )
        );
    }

    out
}

fn metadata_json(metadata: &BenchmarkRunMetadata) -> String {
    let value = serde_json::json!({
        "timestamp": metadata.timestamp,
        "run_directory_name": metadata.run_directory_name,
        "run_index": metadata.run_index,
        "git_commit_hash": metadata.git_commit_hash,
        "git_commit_short_hash": metadata.git_commit_short_hash,
        "ic_testkit_version": metadata.ic_testkit_version,
        "pocket_ic_version": metadata.pocket_ic_version,
        "rustc_version": metadata.rustc_version,
        "benchmark_command": metadata.benchmark_command,
        "selected_previous_run": metadata.selected_previous_run,
    });

    let mut output = serde_json::to_string_pretty(&value).expect("metadata JSON must serialize");
    output.push('\n');
    output
}

fn next_run_index_for_prefix(runs_root: &Path, prefix: &str) -> io::Result<u32> {
    if !runs_root.exists() {
        return Ok(1);
    }

    let mut max_index = 0;
    for entry in fs::read_dir(runs_root)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }

        if let Some(index) = run_index_from_directory_name(&entry.file_name(), prefix) {
            max_index = max_index.max(index);
        }
    }

    Ok(max_index.saturating_add(1))
}

fn run_index_from_directory_name(name: &OsStr, prefix: &str) -> Option<u32> {
    let name = name.to_str()?;
    let index = name.strip_prefix(prefix)?;

    if index.len() == 4 && index.chars().all(|char| char.is_ascii_digit()) {
        index.parse().ok()
    } else {
        None
    }
}

fn short_commit_hash(hash: &str) -> String {
    hash.chars().take(7).collect()
}

fn metadata_required_string(value: &Value, key: &str) -> io::Result<String> {
    metadata_optional_string(value, key)?.ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("missing required metadata string field `{key}`"),
        )
    })
}

fn metadata_required_u32(value: &Value, key: &str) -> io::Result<u32> {
    let Some(raw) = value.get(key).and_then(Value::as_u64) else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("missing required metadata integer field `{key}`"),
        ));
    };

    u32::try_from(raw).map_err(|err| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("metadata integer field `{key}` is invalid: {err}"),
        )
    })
}

fn metadata_optional_string(value: &Value, key: &str) -> io::Result<Option<String>> {
    match value.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(value)) => Ok(Some(value.clone())),
        Some(_) => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("metadata string field `{key}` is not a string or null"),
        )),
    }
}

fn metadata_json_error(err: serde_json::Error) -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        format!("invalid benchmark metadata JSON: {err}"),
    )
}

fn change_suffix(
    comparison: Option<&BenchmarkComparisonRow>,
    change: impl FnOnce(&BenchmarkComparisonRow) -> Option<f64>,
) -> Option<String> {
    comparison.and_then(|row| {
        if row.previous_runs.is_none() {
            Some("new".to_string())
        } else {
            change(row).map(|percent| format!("{percent:+.0}%"))
        }
    })
}

fn format_instructions(value: f64, suffix: Option<String>) -> String {
    with_optional_suffix(format!("{:.4}B", value / 1_000_000_000.0), suffix)
}

fn format_bytes(value: f64, suffix: Option<String>) -> String {
    with_optional_suffix(human_bytes(value), suffix)
}

fn with_optional_suffix(value: String, suffix: Option<String>) -> String {
    match suffix {
        Some(suffix) => format!("{value} ({suffix})"),
        None => value,
    }
}

fn human_bytes(value: f64) -> String {
    const KIB: f64 = 1024.0;
    const MIB: f64 = KIB * 1024.0;
    const GIB: f64 = MIB * 1024.0;

    let (unit_value, unit) = if value.abs() >= GIB {
        (value / GIB, "GB")
    } else if value.abs() >= MIB {
        (value / MIB, "MB")
    } else if value.abs() >= KIB {
        (value / KIB, "KB")
    } else {
        (value, "B")
    };

    format!("{unit_value:+.1} {unit}")
}

const fn kind_str(kind: BenchmarkEventKind) -> &'static str {
    match kind {
        BenchmarkEventKind::Start => "start",
        BenchmarkEventKind::End => "end",
    }
}

fn csv_cell(value: &str) -> String {
    if value.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

fn optional_u64_cell(value: Option<u64>) -> String {
    value.map_or_else(String::new, |value| value.to_string())
}

fn optional_f64_cell(value: Option<f64>) -> String {
    value.map_or_else(String::new, |value| format!("{value:.4}"))
}

fn markdown_cell(value: &str) -> String {
    value.replace('|', "\\|")
}
