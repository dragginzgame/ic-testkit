#!/usr/bin/env python3
"""Opt-in PocketIC 16 fixture benchmark, with sampled Linux process-tree RSS."""

import argparse
import hashlib
import json
import math
import os
from pathlib import Path
import platform
import re
import statistics
import subprocess
import sys
import tempfile
import time

ROOT = Path(__file__).resolve().parents[2]
SAMPLE_INTERVAL = 0.01


def process_snapshot(proc=Path("/proc")):
    """Read process leaders once, excluding threads to avoid duplicate RSS."""
    processes = {}
    page_bytes = os.sysconf("SC_PAGE_SIZE")
    for directory in proc.iterdir():
        if not directory.name.isdecimal():
            continue
        try:
            # comm can contain spaces and ')'; fields after its final ')' are
            # state (3), PPID (4), ..., RSS in pages (24).
            fields = (directory / "stat").read_text().rsplit(")", 1)[1].split()
            processes[int(directory.name)] = (int(fields[1]), int(fields[21]) * page_bytes)
        except (FileNotFoundError, ProcessLookupError, PermissionError):
            continue
    return processes


def tree_rss(root_pid, processes):
    """Sum only the live runner and descendants from the same snapshot."""
    if root_pid not in processes:
        return 0, 0
    children = {}
    for pid, (parent, _) in processes.items():
        children.setdefault(parent, []).append(pid)
    pending = [root_pid]
    seen = set()
    resident = 0
    while pending:
        pid = pending.pop()
        if pid in seen:
            continue
        seen.add(pid)
        resident += processes[pid][1]
        pending.extend(children.get(pid, []))
    return resident, len(seen)


def measure(command):
    # A file avoids pipe backpressure when the raw sample report exceeds a pipe
    # buffer. The process context waits for RAII server cleanup on exceptions.
    with tempfile.TemporaryFile() as output, subprocess.Popen(
        command, stdout=output, cwd=ROOT, start_new_session=True
    ) as process:
        peak_rss = 0
        peak_processes = 0
        count = 0
        while process.poll() is None:
            resident, processes = tree_rss(process.pid, process_snapshot())
            peak_rss = max(peak_rss, resident)
            peak_processes = max(peak_processes, processes)
            count += 1
            time.sleep(SAMPLE_INTERVAL)
        if process.returncode:
            raise RuntimeError(f"benchmark worker exited with {process.returncode}")
        output.seek(0)
        result = json.load(output)
    result["sampled_peak_tree_rss_bytes"] = peak_rss
    result["peak_tree_process_count"] = peak_processes
    result["rss_samples"] = count
    return result


def distribution(values):
    ordered = sorted(values)
    if not ordered:
        return None
    return {
        "count": len(ordered),
        "mean": statistics.fmean(ordered),
        "p50": statistics.median(ordered),
        "p95": ordered[math.ceil(len(ordered) * 0.95) - 1],
    }


def summarize(runs):
    samples = [sample for run in runs for sample in run["samples"]]
    phases = {}
    for phase in samples[0]["phases"]:
        phases[phase] = distribution(
            sample["phases"][phase]
            for sample in samples
            if sample["phases"][phase] is not None
        )
    return {
        "phases_ms": phases,
        "body_ms": distribution(sample["body_ms"] for sample in samples),
        "release_ms": distribution(sample["release_ms"] for sample in samples),
        "iteration_ms": distribution(sample["total_ms"] for sample in samples),
        "work_wall_ms": distribution(run["wall_ms"] for run in runs),
        "fixture_wall_ms": distribution(run["fixture_wall_ms"] for run in runs),
        "iterations_per_second": distribution(
            run["iterations"] * 1000 / run["wall_ms"] for run in runs
        ),
        "sampled_peak_tree_rss_bytes": distribution(
            run["sampled_peak_tree_rss_bytes"] for run in runs
        ),
    }


def positive(value):
    number = int(value)
    if number < 1:
        raise argparse.ArgumentTypeError("must be positive")
    return number


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--server", type=Path, required=True, help="exact PocketIC 16 binary")
    parser.add_argument("--iterations", type=positive, default=100, help="total tasks per mode/run")
    parser.add_argument("--workers", type=positive, default=2, help="same concurrency in all modes")
    parser.add_argument("--repeats", type=positive, default=3)
    parser.add_argument(
        "--modes", nargs="+", choices=["fresh", "pooled-1", "pooled-2"],
        default=["fresh", "pooled-1", "pooled-2"],
        help="modes to compare; select pooled modes to skip the fresh-fixture control",
    )
    parser.add_argument("--state-bytes", type=int, default=1024 * 1024, help="heap bytes per canister")
    parser.add_argument("--profile", choices=["release", "dev"], default="release")
    parser.add_argument("--output", type=Path, help="save provenance, raw samples, and summaries as JSON")
    args = parser.parse_args()
    if not sys.platform.startswith("linux"):
        parser.error("process-tree RSS sampling requires Linux /proc")
    if args.workers > args.iterations or not 0 <= args.state_bytes <= 0xFFFFFFFF:
        parser.error("workers must not exceed iterations; state-bytes must fit u32")
    if len(set(args.modes)) != len(args.modes):
        parser.error("each mode must be selected only once")
    server = args.server.resolve(strict=True)
    version = subprocess.check_output([server, "--version"], text=True).strip()
    if not re.fullmatch(r"pocket-ic-server 16\.\d+\.\d+", version):
        parser.error(f"expected PocketIC 16, got {version!r}")
    metadata = json.loads(subprocess.check_output(
        ["cargo", "metadata", "--locked", "--no-deps", "--format-version", "1"], cwd=ROOT
    ))
    target = Path(metadata["target_directory"])
    subprocess.run([
        "cargo", "build", "--locked", "-p", "ic-testkit", "--profile", args.profile,
        "--example", "fixture_reuse_benchmark",
    ], cwd=ROOT, check=True)
    subprocess.run([
        "cargo", "build", "--locked", "--target", "wasm32-unknown-unknown",
        "-p", "ic_testkit_perf_probe",
    ], cwd=ROOT, check=True)
    worker = target / ("debug" if args.profile == "dev" else args.profile) / "examples/fixture_reuse_benchmark"
    wasm = target / "wasm32-unknown-unknown/debug/ic_testkit_perf_probe.wasm"
    modes = {"fresh": ("fresh", args.workers), "pooled-1": ("pooled", 1), "pooled-2": ("pooled", 2)}
    runs = {label: [] for label in args.modes}
    for repeat in range(args.repeats):
        # Rotate mode order to avoid always giving one mode the first run.
        offset = repeat % len(args.modes)
        for label in args.modes[offset:] + args.modes[:offset]:
            mode, capacity = modes[label]
            print(f"Run {repeat + 1}/{args.repeats}: {label}", file=sys.stderr, flush=True)
            runs[label].append(measure([
                str(worker), mode, str(capacity), str(args.iterations), str(args.workers),
                str(args.state_bytes), str(wasm), str(server),
            ]))
    report = {
        "format": "ic-testkit-fixture-benchmark-v1",
        "provenance": {
            "server": str(server), "server_version": version,
            "platform": platform.platform(), "logical_cpus": os.cpu_count(),
            "rustc": subprocess.check_output(["rustc", "--version"], text=True).strip(),
            "revision": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip(),
            "working_tree_dirty": bool(subprocess.check_output(["git", "status", "--porcelain"], cwd=ROOT)),
            "host_profile": args.profile, "wasm_profile": "dev",
            "wasm_sha256": hashlib.sha256(wasm.read_bytes()).hexdigest(),
            "rss_sample_interval_ms": SAMPLE_INTERVAL * 1000,
            "iterations": args.iterations, "workers": args.workers,
            "repeats": args.repeats, "state_bytes_per_canister": args.state_bytes,
            "modes": args.modes,
        },
        "runs": runs,
        "summary": {label: summarize(items) for label, items in runs.items()},
    }
    if args.output:
        args.output.write_text(json.dumps(report, indent=2) + "\n")
    print("Mode        work ms   incl. setup/drop ms   tasks/s   restore mean ms   wait mean ms   peak RSS MiB")
    for label, summary in report["summary"].items():
        phases = summary["phases_ms"]
        restore = phases["restore_ms"]
        restore_text = f"{restore['mean']:.2f}" if restore else "-"
        print(
            f"{label:10} {summary['work_wall_ms']['p50']:9.2f}"
            f" {summary['fixture_wall_ms']['p50']:21.2f}"
            f" {summary['iterations_per_second']['p50']:9.2f}"
            f" {restore_text:>17} {phases['wait_ms']['mean']:14.2f}"
            f" {summary['sampled_peak_tree_rss_bytes']['p50'] / 1024**2:14.2f}"
        )


if __name__ == "__main__":
    main()
