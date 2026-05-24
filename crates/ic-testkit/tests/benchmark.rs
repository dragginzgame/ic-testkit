use ic_testkit::benchmark::{
    BenchmarkAggregateReport, BenchmarkCounters, BenchmarkEventKind, BenchmarkEventSource,
    BenchmarkParseReport, BenchmarkParserConfig, BenchmarkRunMetadata, BenchmarkRunReport,
    BenchmarkSpanReport, DEFAULT_PREFIX, aggregate_benchmark_spans, benchmark_run_directory_name,
    compare_benchmark_aggregates, find_latest_previous_run, format_marker,
    next_benchmark_run_directory, pair_benchmark_spans, parse_benchmark_events,
    parse_benchmark_events_from_captured_output, parse_benchmark_events_from_source,
    read_benchmark_run_metadata, write_benchmark_report_dir,
};
use std::{fs, path::PathBuf};

#[test]
fn parses_compact_markers_and_ignores_non_marker_lines() {
    let input = "\
ordinary debug line
ICTK|app/myfunc/something:start|100|200|300|400
ICTK|app/myfunc/something:end|150|260|390|430
";

    let report = parse_benchmark_events(input, &BenchmarkParserConfig::default());

    assert_eq!(report.ignored_line_count, 1);
    assert!(report.malformed_markers.is_empty());
    assert_eq!(report.events.len(), 2);
    assert_eq!(report.events[0].suite, "app");
    assert_eq!(report.events[0].span_label, "app/myfunc/something");
    assert_eq!(report.events[0].kind, BenchmarkEventKind::Start);
    assert_eq!(report.events[1].kind, BenchmarkEventKind::End);
}

#[test]
fn marker_formatter_uses_compact_tuple_shape() {
    let line = format_marker(
        DEFAULT_PREFIX,
        "app/myfunc/something:start",
        BenchmarkCounters {
            instructions: 1,
            heap_bytes: 2,
            memory_bytes: 3,
            total_allocation: 4,
        },
    );

    assert_eq!(line, "ICTK|app/myfunc/something:start|1|2|3|4");
}

#[test]
fn records_source_stream_for_stdout_and_stderr_capture() {
    let stdout = parse_benchmark_events_from_source(
        "ICTK|app/a:start|1|2|3|4",
        &BenchmarkParserConfig::default(),
        BenchmarkEventSource::Stdout,
    );
    let stderr = parse_benchmark_events_from_source(
        "ICTK|app/a:end|2|3|4|5",
        &BenchmarkParserConfig::default(),
        BenchmarkEventSource::Stderr,
    );

    assert_eq!(stdout.events[0].source, BenchmarkEventSource::Stdout);
    assert_eq!(stderr.events[0].source, BenchmarkEventSource::Stderr);
}

#[test]
fn parses_combined_captured_stdout_and_stderr() {
    let report = parse_benchmark_events_from_captured_output(
        "noise\nICTK|app/a:start|1|2|3|4\n",
        "ICTK|app/a:end|5|6|7|8\n",
        &BenchmarkParserConfig::default(),
    );

    assert_eq!(report.ignored_line_count, 1);
    assert_eq!(report.events.len(), 2);
    assert_eq!(report.events[0].source, BenchmarkEventSource::Stdout);
    assert_eq!(report.events[1].source, BenchmarkEventSource::Stderr);
}

#[test]
fn malformed_markers_capture_bad_shape_and_bad_numbers() {
    let input = "\
ICTK|app/a:start|1|2|3
ICTK|app/a:end|1|two|3|4
ICTK||1|2|3|4
";

    let report = parse_benchmark_events(input, &BenchmarkParserConfig::default());

    assert!(report.events.is_empty());
    assert_eq!(report.malformed_markers.len(), 3);
    assert!(
        report
            .malformed_markers
            .iter()
            .any(|marker| marker.reason.contains("six"))
    );
    assert!(
        report
            .malformed_markers
            .iter()
            .any(|marker| marker.reason.contains("unsigned integer"))
    );
    assert!(
        report
            .malformed_markers
            .iter()
            .any(|marker| marker.reason.contains("label is empty"))
    );
}

#[test]
fn strict_parser_reports_non_marker_log_lines() {
    let config = BenchmarkParserConfig {
        strict: true,
        ..BenchmarkParserConfig::default()
    };
    let report = parse_benchmark_events(
        "\
ordinary debug line
ICTK|app/a:start|1|2|3|4
",
        &config,
    );

    assert_eq!(report.events.len(), 1);
    assert_eq!(report.ignored_line_count, 0);
    assert_eq!(report.malformed_markers.len(), 1);
    assert_eq!(
        report.malformed_markers[0].reason,
        "line does not use a configured marker prefix"
    );
}

#[test]
fn pairs_repeated_same_label_spans_by_stack_order() {
    let report = parse_benchmark_events(
        "\
ICTK|app/a:start|10|10|10|10
ICTK|app/a:start|20|20|20|20
ICTK|app/a:end|30|30|30|30
ICTK|app/a:end|50|50|50|50
",
        &BenchmarkParserConfig::default(),
    );

    let spans = pair_benchmark_spans(&report.events);

    assert_eq!(spans.spans.len(), 2);
    assert!(spans.unpaired_markers.is_empty());
    assert_eq!(spans.spans[0].start_line, 2);
    assert_eq!(spans.spans[0].end_line, 3);
    assert_eq!(spans.spans[0].delta.instructions, 10);
    assert_eq!(spans.spans[1].start_line, 1);
    assert_eq!(spans.spans[1].end_line, 4);
    assert_eq!(spans.spans[1].delta.instructions, 40);
}

#[test]
fn reports_unpaired_and_invalid_spans() {
    let report = parse_benchmark_events(
        "\
ICTK|app/a:end|10|10|10|10
ICTK|app/b:start|20|20|20|20
ICTK|app/c:start|50|50|50|50
ICTK|app/c:end|40|60|60|60
",
        &BenchmarkParserConfig::default(),
    );

    let spans = pair_benchmark_spans(&report.events);

    assert!(spans.spans.is_empty());
    assert_eq!(spans.unpaired_markers.len(), 2);
    assert_eq!(spans.invalid_spans.len(), 1);
    assert_eq!(
        spans.invalid_spans[0].reason,
        "end counter is lower than start counter"
    );
}

#[test]
fn aggregates_by_suite_and_all_suites() {
    let parse = parse_benchmark_events(
        "\
ICTK|app/a:start|0|0|0|0
ICTK|app/a:end|100|20|30|40
ICTK|app/a:start|100|20|30|40
ICTK|app/a:end|300|60|90|120
",
        &BenchmarkParserConfig::default(),
    );
    let spans = pair_benchmark_spans(&parse.events);
    let aggregates = aggregate_benchmark_spans(&spans.spans);

    let app = aggregates
        .rows
        .iter()
        .find(|row| row.suite == "app" && row.span_label == "app/a")
        .expect("app aggregate");
    let all = aggregates
        .rows
        .iter()
        .find(|row| row.suite == "ALL" && row.span_label == "app/a")
        .expect("ALL aggregate");

    assert_eq!(app.runs, 2);
    assert_eq!(app.total.instructions, 300);
    assert!((app.average.instructions - 150.0).abs() < f64::EPSILON);
    assert_eq!(app.min.instructions, 100);
    assert_eq!(app.max.instructions, 200);
    assert_eq!(all.runs, 2);
}

#[test]
fn compares_current_average_against_previous_average() {
    let previous = aggregate_benchmark_spans(
        &pair_benchmark_spans(
            &parse_benchmark_events(
                "\
ICTK|app/a:start|0|0|0|0
ICTK|app/a:end|100|100|100|100
",
                &BenchmarkParserConfig::default(),
            )
            .events,
        )
        .spans,
    );
    let current = aggregate_benchmark_spans(
        &pair_benchmark_spans(
            &parse_benchmark_events(
                "\
ICTK|app/a:start|0|0|0|0
ICTK|app/a:end|150|50|100|100
",
                &BenchmarkParserConfig::default(),
            )
            .events,
        )
        .spans,
    );

    let comparison = compare_benchmark_aggregates(&current.rows, &previous.rows);
    let app = comparison
        .rows
        .iter()
        .find(|row| row.suite == "app" && row.span_label == "app/a")
        .expect("comparison row");

    assert_eq!(app.instructions_avg_change_percent, Some(50.0));
    assert_eq!(app.heap_bytes_avg_change_percent, Some(-50.0));
}

#[test]
fn writes_csv_markdown_and_metadata_report_files() {
    let parse = parse_benchmark_events(
        "\
ICTK|app/myfunc/something:start|0|0|0|0
ICTK|app/myfunc/something:end|234_200_000|1|4_194_304|2
"
        .replace('_', "")
        .as_str(),
        &BenchmarkParserConfig::default(),
    );
    let spans = pair_benchmark_spans(&parse.events);
    let aggregates = aggregate_benchmark_spans(&spans.spans);
    let previous_parse = parse_benchmark_events(
        "\
ICTK|app/myfunc/something:start|0|0|0|0
ICTK|app/myfunc/something:end|174776119|1|5447148|2
",
        &BenchmarkParserConfig::default(),
    );
    let previous_spans = pair_benchmark_spans(&previous_parse.events);
    let previous_aggregates = aggregate_benchmark_spans(&previous_spans.spans);
    let comparison = compare_benchmark_aggregates(&aggregates.rows, &previous_aggregates.rows);
    let report = BenchmarkRunReport {
        parse,
        spans,
        aggregates,
        comparison: Some(comparison),
        metadata: BenchmarkRunMetadata {
            timestamp: "2026-05-24T162600Z".to_string(),
            run_directory_name: "2026-05-24T162600Z-a1b2c3d-0001".to_string(),
            run_index: 1,
            git_commit_hash: Some("a1b2c3d4".to_string()),
            git_commit_short_hash: Some("a1b2c3d".to_string()),
            ic_testkit_version: env!("CARGO_PKG_VERSION").to_string(),
            pocket_ic_version: "13.0".to_string(),
            rustc_version: "rustc 1.88.0".to_string(),
            benchmark_command: Some("make \"test\"\nagain".to_string()),
            selected_previous_run: None,
        },
    };
    let root = unique_temp_dir("ic-testkit-benchmark-report");

    write_benchmark_report_dir(&report, &root).expect("write report");

    assert!(root.join("raw-events.csv").exists());
    assert!(root.join("spans.csv").exists());
    assert!(root.join("suite-aggregates.csv").exists());
    assert!(root.join("all-aggregates.csv").exists());
    assert!(root.join("bench-summary.md").exists());
    assert!(root.join("metadata.json").exists());

    let summary = fs::read_to_string(root.join("bench-summary.md")).expect("read summary");
    assert!(summary.contains("| app/myfunc/something | 1 | 0.2342B (+34%)"));
    assert!(summary.contains("+4.0 MB (-23%)"));

    let metadata = fs::read_to_string(root.join("metadata.json")).expect("read metadata");
    assert!(metadata.contains("\"run_directory_name\": \"2026-05-24T162600Z-a1b2c3d-0001\""));
    let read_metadata =
        read_benchmark_run_metadata(root.join("metadata.json")).expect("round-trip metadata");
    assert_eq!(
        read_metadata.benchmark_command.as_deref(),
        Some("make \"test\"\nagain")
    );

    fs::remove_dir_all(root).expect("clean temp dir");
}

#[test]
fn run_directory_helpers_choose_next_index_for_commit_and_timestamp() {
    let root = unique_temp_dir("ic-testkit-benchmark-runs");
    fs::create_dir_all(root.join("2026-05-24T162600Z-a1b2c3d-0001")).expect("first dir");
    fs::create_dir_all(root.join("2026-05-24T162600Z-a1b2c3d-0002")).expect("second dir");
    fs::create_dir_all(root.join("2026-05-24T162600Z-deadbee-0009")).expect("other commit dir");

    let next = next_benchmark_run_directory(&root, "2026-05-24T162600Z", Some("a1b2c3d4e5f6"))
        .expect("next run dir");

    assert_eq!(
        benchmark_run_directory_name("2026-05-24T162600Z", Some("a1b2c3d"), 3),
        "2026-05-24T162600Z-a1b2c3d-0003"
    );
    assert_eq!(next.directory_name, "2026-05-24T162600Z-a1b2c3d-0003");
    assert_eq!(next.run_index, 3);
    assert_eq!(next.git_commit_short_hash.as_deref(), Some("a1b2c3d"));

    fs::remove_dir_all(root).expect("clean temp dir");
}

#[test]
fn run_directory_helper_uses_unknown_commit_segment_without_git_metadata() {
    let root = unique_temp_dir("ic-testkit-benchmark-runs-unknown");
    let next =
        next_benchmark_run_directory(&root, "2026-05-24T162600Z", None).expect("next run dir");

    assert_eq!(next.directory_name, "2026-05-24T162600Z-unknown-0001");
    assert_eq!(next.git_commit_hash, None);
    assert_eq!(next.git_commit_short_hash, None);

    fs::remove_dir_all(root).expect("clean temp dir");
}

#[test]
fn previous_run_discovery_selects_latest_matching_metadata() {
    let root = unique_temp_dir("ic-testkit-benchmark-previous");
    let older = root.join("2026-05-24T100000Z-a1b2c3d-0001");
    let latest = root.join("2026-05-24T110000Z-a1b2c3d-0001");
    let other_command = root.join("2026-05-24T120000Z-a1b2c3d-0001");
    fs::create_dir_all(&older).expect("older dir");
    fs::create_dir_all(&latest).expect("latest dir");
    fs::create_dir_all(&other_command).expect("other command dir");
    write_metadata(
        &older,
        "2026-05-24T100000Z",
        "2026-05-24T100000Z-a1b2c3d-0001",
        Some("suite-a"),
    );
    write_metadata(
        &latest,
        "2026-05-24T110000Z",
        "2026-05-24T110000Z-a1b2c3d-0001",
        Some("suite-a"),
    );
    write_metadata(
        &other_command,
        "2026-05-24T120000Z",
        "2026-05-24T120000Z-a1b2c3d-0001",
        Some("suite-b"),
    );

    let previous =
        find_latest_previous_run(&root, "2026-05-24T130000Z-a1b2c3d-0001", Some("suite-a"))
            .expect("find previous")
            .expect("previous run");

    assert_eq!(previous, latest);

    fs::remove_dir_all(root).expect("clean temp dir");
}

#[test]
fn metadata_reader_round_trips_written_metadata() {
    let root = unique_temp_dir("ic-testkit-benchmark-metadata");
    write_metadata(
        &root,
        "2026-05-24T162600Z",
        "2026-05-24T162600Z-a1b2c3d-0001",
        Some("make benchmark"),
    );

    let metadata = read_benchmark_run_metadata(root.join("metadata.json")).expect("read metadata");

    assert_eq!(metadata.timestamp, "2026-05-24T162600Z");
    assert_eq!(
        metadata.run_directory_name,
        "2026-05-24T162600Z-a1b2c3d-0001"
    );
    assert_eq!(metadata.run_index, 1);
    assert_eq!(
        metadata.benchmark_command.as_deref(),
        Some("make benchmark")
    );

    fs::remove_dir_all(root).expect("clean temp dir");
}

fn write_metadata(
    path: &std::path::Path,
    timestamp: &str,
    run_directory_name: &str,
    command: Option<&str>,
) {
    let metadata = BenchmarkRunMetadata {
        timestamp: timestamp.to_string(),
        run_directory_name: run_directory_name.to_string(),
        run_index: 1,
        git_commit_hash: Some("a1b2c3d4".to_string()),
        git_commit_short_hash: Some("a1b2c3d".to_string()),
        ic_testkit_version: env!("CARGO_PKG_VERSION").to_string(),
        pocket_ic_version: "13.0".to_string(),
        rustc_version: "rustc 1.88.0".to_string(),
        benchmark_command: command.map(str::to_string),
        selected_previous_run: None,
    };
    let report = BenchmarkRunReport {
        parse: BenchmarkParseReport::default(),
        spans: BenchmarkSpanReport::default(),
        aggregates: BenchmarkAggregateReport::default(),
        comparison: None,
        metadata,
    };

    write_benchmark_report_dir(&report, path).expect("write metadata fixture");
}

fn unique_temp_dir(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("{name}-{}", std::process::id()));
    if root.exists() {
        fs::remove_dir_all(&root).expect("remove stale temp dir");
    }
    fs::create_dir_all(&root).expect("create temp dir");
    root
}
