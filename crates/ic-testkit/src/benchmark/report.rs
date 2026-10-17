//! Native benchmark metadata, CSV and Markdown report output.
//!
//! Paths and synchronization remain caller-owned; retained report formats are
//! unchanged. Marker parsing and arithmetic live in the parent module.

use std::{fmt::Write as _, fs, io, path::Path};

use super::{
    BenchmarkAggregateRow, BenchmarkAverages, BenchmarkComparisonRow, BenchmarkEventKind,
    BenchmarkRunMetadata, BenchmarkRunReport, BenchmarkScope, BenchmarkSpan, InvalidBenchmarkSpan,
    MalformedBenchmarkMarker, RawBenchmarkEvent,
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

pub fn read_benchmark_run_metadata(path: impl AsRef<Path>) -> io::Result<BenchmarkRunMetadata> {
    let input = fs::read_to_string(path)?;
    if !input.trim_start().starts_with('{') {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "benchmark metadata JSON must be an object",
        ));
    }
    serde_json::from_str(&input).map_err(metadata_json_error)
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
            .iter()
            .filter(|row| matches!(row.scope, BenchmarkScope::Suite(_))),
    )?;
    let all_aggregates = aggregates_csv(
        report
            .aggregates
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
        comparison_csv(report.comparison.as_deref()),
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
            csv_cell(&format!("{}:{}", event.span_label, kind_str(event.kind))),
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

fn unpaired_markers_csv(markers: &[RawBenchmarkEvent]) -> String {
    let mut out = String::from("source_line,source,kind,suite,span_label,label\n");
    for marker in markers {
        let kind = kind_str(marker.kind);
        let _ = writeln!(
            out,
            "{},{},{},{},{},{}",
            marker.source_line,
            marker.source.as_str(),
            kind,
            csv_cell(&marker.suite),
            csv_cell(&marker.span_label),
            csv_cell(&format!("{}:{kind}", marker.span_label))
        );
    }
    out
}

fn invalid_spans_csv(spans: &[InvalidBenchmarkSpan]) -> String {
    let mut out = String::from("start_line,end_line,suite,span_label,reason\n");
    for span in spans {
        let _ = writeln!(
            out,
            "{},{},{},{},end counter is lower than start counter",
            span.start.source_line,
            span.end.source_line,
            csv_cell(&span.start.suite),
            csv_cell(&span.start.span_label)
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

fn comparison_csv(comparison: Option<&[BenchmarkComparisonRow]>) -> String {
    let mut out = String::from(
        "scope,suite,span_label,current_runs,previous_runs,current_mean_scale,previous_mean_scale,current_instructions_mean,previous_instructions_mean,current_heap_bytes_mean,previous_heap_bytes_mean,current_memory_bytes_mean,previous_memory_bytes_mean,current_total_allocation_mean,previous_total_allocation_mean\n",
    );
    if let Some(comparison) = comparison {
        for row in comparison {
            let current = row.current.map(|measurement| measurement.average);
            let previous = row.previous.map(|measurement| measurement.average);
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
                optional_u64_cell(row.current.map(|measurement| measurement.runs)),
                optional_u64_cell(row.previous.map(|measurement| measurement.runs)),
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
