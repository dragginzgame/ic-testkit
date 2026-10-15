//! Native benchmark report directories, metadata, CSV and Markdown output.
//!
//! Paths and synchronization remain caller-owned; retained report formats are
//! unchanged. Marker parsing and arithmetic live in the parent module.

use std::{
    fmt::Write as _,
    fs, io,
    path::{Path, PathBuf},
};

use serde_json::Value;

use super::{
    BenchmarkAggregateRow, BenchmarkAverages, BenchmarkComparisonReport, BenchmarkEventKind,
    BenchmarkRunMetadata, BenchmarkRunReport, BenchmarkScope, BenchmarkSpan, InvalidBenchmarkSpan,
    MalformedBenchmarkMarker, RawBenchmarkEvent, UnpairedBenchmarkMarker,
    UnpairedBenchmarkMarkerKind,
};

const ALL_SUITES_LABEL: &str = "ALL";

impl BenchmarkScope {
    fn label(&self) -> &str {
        match self {
            Self::Suite(suite) => suite,
            Self::All => ALL_SUITES_LABEL,
        }
    }
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

/// Compute the next available benchmark run path for one timestamp and commit.
///
/// This function does not create or reserve the returned directory. Callers
/// allocating the same prefix concurrently must synchronize that shared
/// resource or provide unique timestamps.
/// A missing runs root starts at index one; other directory errors are returned.
pub fn next_benchmark_run_directory(
    runs_root: impl AsRef<Path>,
    timestamp: &str,
    git_commit_hash: Option<&str>,
) -> io::Result<BenchmarkRunDirectory> {
    let runs_root = runs_root.as_ref();
    let git_commit_hash = git_commit_hash.filter(|hash| !hash.is_empty());
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

/// Find the latest earlier run with readable metadata and a matching command.
///
/// Names must follow [`benchmark_run_directory_name`]; indices are ordered
/// numerically even when they exceed four digits. Eligible runs are ranked by
/// metadata timestamp, directory prefix, then numeric index. An invalid current
/// name returns [`io::ErrorKind::InvalidInput`]. Unreadable or malformed
/// candidates are skipped.
/// A missing runs root has no previous run; other directory errors are returned.
pub fn find_latest_previous_run(
    runs_root: impl AsRef<Path>,
    current_run_directory_name: &str,
    benchmark_command: Option<&str>,
) -> io::Result<Option<PathBuf>> {
    let runs_root = runs_root.as_ref();
    let current_key = run_directory_sort_key(current_run_directory_name).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "invalid benchmark run directory name",
        )
    })?;
    let mut latest: Option<((String, String, u32), PathBuf)> = None;

    let entries = match fs::read_dir(runs_root) {
        Ok(entries) => entries,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error),
    };
    for entry in entries {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }

        let directory_name = entry.file_name().to_string_lossy().into_owned();
        let Some((prefix, index)) = run_directory_sort_key(&directory_name) else {
            continue;
        };
        if (prefix, index) >= current_key {
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

        let key = (metadata.timestamp, prefix.to_owned(), index);
        if latest
            .as_ref()
            .is_none_or(|(latest_key, _)| key >= *latest_key)
        {
            latest = Some((key, entry.path()));
        }
    }

    Ok(latest.map(|(_, path)| path))
}

pub fn read_benchmark_run_metadata(path: impl AsRef<Path>) -> io::Result<BenchmarkRunMetadata> {
    let input = fs::read_to_string(path)?;
    let value = serde_json::from_str::<Value>(&input).map_err(metadata_json_error)?;

    if !value.is_object() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "benchmark metadata JSON must be an object",
        ));
    }
    serde_json::from_value(value).map_err(metadata_json_error)
}

/// Write one complete benchmark report into the requested directory.
///
/// The directory is caller-owned. Concurrent writers must use unique paths or
/// synchronize access to the same path outside this function.
pub fn write_benchmark_report_dir(
    report: &BenchmarkRunReport,
    path: impl AsRef<Path>,
) -> io::Result<()> {
    let path = path.as_ref();
    let suite_aggregates = aggregates_csv(
        report
            .aggregates
            .rows
            .iter()
            .filter(|row| matches!(row.scope, BenchmarkScope::Suite(_))),
    )?;
    let all_aggregates = aggregates_csv(
        report
            .aggregates
            .rows
            .iter()
            .filter(|row| matches!(row.scope, BenchmarkScope::All)),
    )?;
    let summary = benchmark_summary_markdown(report)?;
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
    fs::write(path.join("suite-aggregates.csv"), suite_aggregates)?;
    fs::write(path.join("all-aggregates.csv"), all_aggregates)?;
    fs::write(
        path.join("comparison.csv"),
        comparison_csv(report.comparison.as_ref()),
    )?;
    fs::write(path.join("bench-summary.md"), summary)?;
    fs::write(path.join("metadata.json"), metadata_json(&report.metadata))?;

    Ok(())
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

fn aggregates_csv<'a>(rows: impl Iterator<Item = &'a BenchmarkAggregateRow>) -> io::Result<String> {
    let mut out = String::from(
        "suite,span_label,runs,instructions_total,instructions_mean,heap_bytes_total,heap_bytes_mean,memory_bytes_total,memory_bytes_mean,total_allocation_total,total_allocation_mean\n",
    );
    for row in rows {
        let average = row.average(1).map_err(io::Error::other)?;
        let _ = writeln!(
            out,
            "{},{},{},{},{},{},{},{},{},{},{}",
            csv_cell(row.scope.label()),
            csv_cell(&row.span_label),
            row.runs,
            row.total.instructions,
            average.instructions,
            row.total.heap_bytes,
            average.heap_bytes,
            row.total.memory_bytes,
            average.memory_bytes,
            row.total.total_allocation,
            average.total_allocation
        );
    }
    Ok(out)
}

fn comparison_csv(comparison: Option<&BenchmarkComparisonReport>) -> String {
    let mut out = String::from(
        "scope,suite,span_label,current_runs,previous_runs,current_mean_scale,previous_mean_scale,current_instructions_mean,previous_instructions_mean,current_heap_bytes_mean,previous_heap_bytes_mean,current_memory_bytes_mean,previous_memory_bytes_mean,current_total_allocation_mean,previous_total_allocation_mean\n",
    );
    if let Some(comparison) = comparison {
        for row in &comparison.rows {
            let current = row.current_average;
            let previous = row.previous_average;
            let value = |mean: Option<BenchmarkAverages>, field: fn(BenchmarkAverages) -> u128| {
                mean.map_or_else(String::new, |mean| field(mean).to_string())
            };
            let _ = writeln!(
                out,
                "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
                if matches!(row.scope, BenchmarkScope::All) {
                    "all"
                } else {
                    "suite"
                },
                csv_cell(row.scope.label()),
                csv_cell(&row.span_label),
                optional_u64_cell(row.current_runs),
                optional_u64_cell(row.previous_runs),
                current.map_or_else(String::new, |mean| mean.scale.to_string()),
                previous.map_or_else(String::new, |mean| mean.scale.to_string()),
                value(current, |m| m.instructions),
                value(previous, |m| m.instructions),
                value(current, |m| m.heap_bytes),
                value(previous, |m| m.heap_bytes),
                value(current, |m| m.memory_bytes),
                value(previous, |m| m.memory_bytes),
                value(current, |m| m.total_allocation),
                value(previous, |m| m.total_allocation)
            );
        }
    }
    out
}

fn benchmark_summary_markdown(report: &BenchmarkRunReport) -> io::Result<String> {
    let mut out = String::from(
        "# Benchmark Summary\n\nMeans are floor-rounded in original counter units. Comparison CSV retains current and previous scaled means.\n\n| Benchmark | Runs | Instructions Mean | Heap Bytes Mean | Memory Bytes Mean | Allocation Mean |\n| --- | ---: | ---: | ---: | ---: | ---: |\n",
    );
    for row in report
        .aggregates
        .rows
        .iter()
        .filter(|row| matches!(row.scope, BenchmarkScope::Suite(_)))
    {
        let average = row.average(1).map_err(io::Error::other)?;
        let _ = writeln!(
            out,
            "| {} | {} | {} | {} | {} | {} |",
            markdown_cell(&row.span_label),
            row.runs,
            average.instructions,
            average.heap_bytes,
            average.memory_bytes,
            average.total_allocation
        );
    }
    Ok(out)
}

fn metadata_json(metadata: &BenchmarkRunMetadata) -> String {
    let mut output = serde_json::to_string_pretty(metadata).expect("metadata JSON must serialize");
    output.push('\n');
    output
}

fn next_run_index_for_prefix(runs_root: &Path, prefix: &str) -> io::Result<u32> {
    let entries = match fs::read_dir(runs_root) {
        Ok(entries) => entries,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(1),
        Err(error) => return Err(error),
    };
    let mut max_index = 0;
    for entry in entries {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }

        if let Some(index) = entry
            .file_name()
            .to_str()
            .and_then(|name| name.strip_prefix(prefix))
            .and_then(parse_run_index)
        {
            max_index = max_index.max(index);
        }
    }

    max_index
        .checked_add(1)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "benchmark run index exhausted"))
}

fn parse_run_index(index: &str) -> Option<u32> {
    if index.len() >= 4 && index.chars().all(|char| char.is_ascii_digit()) {
        index.parse().ok()
    } else {
        None
    }
}

fn run_directory_sort_key(name: &str) -> Option<(&str, u32)> {
    let (prefix, index) = name.rsplit_once('-')?;
    Some((prefix, parse_run_index(index)?))
}

fn short_commit_hash(hash: &str) -> String {
    hash.chars().take(7).collect()
}

fn metadata_json_error(err: serde_json::Error) -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        format!("invalid benchmark metadata JSON: {err}"),
    )
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

fn markdown_cell(value: &str) -> String {
    value.replace('|', "\\|")
}
