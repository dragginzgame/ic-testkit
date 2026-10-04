#!/usr/bin/env python3
"""Focused measurement-boundary checks; no PocketIC or Cargo work."""

from contextlib import redirect_stderr
import io
from pathlib import Path
import runpy
import tempfile
import unittest

BENCH = runpy.run_path(str(Path(__file__).with_name("benchmark-fixture-reuse.py")))


class CapacitySelectionTests(unittest.TestCase):
    def test_capacity_sweep_preserves_order_and_one_fresh_control(self):
        args = BENCH["parse_arguments"]([
            "--server", "/unused/pocket-ic", "--iterations", "8", "--workers", "8",
            "--modes", "pooled", "fresh", "--capacities", "1", "2", "4", "8",
        ])
        self.assertEqual(BENCH["benchmark_cases"](args), [
            ("pooled-1", "pooled", 1), ("pooled-2", "pooled", 2),
            ("pooled-4", "pooled", 4), ("pooled-8", "pooled", 8),
            ("fresh", "fresh", 8),
        ])

    def test_default_and_fresh_only_selection(self):
        args = BENCH["parse_arguments"](["--server", "/unused/pocket-ic"])
        self.assertEqual(BENCH["benchmark_cases"](args), [
            ("fresh", "fresh", 2), ("pooled-1", "pooled", 1), ("pooled-2", "pooled", 2),
        ])
        args = BENCH["parse_arguments"]([
            "--server", "/unused/pocket-ic", "--modes", "fresh", "--capacities", "8",
        ])
        self.assertEqual(BENCH["benchmark_cases"](args), [("fresh", "fresh", 2)])

    def test_invalid_selections_fail_before_launching_tools(self):
        for selection in [
            ["--capacities", "0"], ["--capacities", "-1"],
            ["--capacities", "2", "2"], ["--modes", "pooled", "pooled"],
        ]:
            with self.subTest(selection=selection), redirect_stderr(io.StringIO()):
                with self.assertRaises(SystemExit) as error:
                    BENCH["parse_arguments"](["--server", "/unused/pocket-ic", *selection])
                self.assertEqual(error.exception.code, 2)


class MeasurementTests(unittest.TestCase):
    def test_tree_counts_runner_and_descendants_but_not_parent_or_sibling(self):
        processes = {
            1: (0, 10000), 10: (1, 100), 11: (1, 20000),
            20: (10, 200), 21: (20, 300), 22: (10, 400),
        }
        self.assertEqual(BENCH["tree_rss"](10, processes), (1000, 4))
        self.assertEqual(BENCH["tree_rss"](999, processes), (0, 0))

    def test_kernel_stat_with_parentheses_in_comm_preserves_parent_and_rss(self):
        with tempfile.TemporaryDirectory() as root:
            process = Path(root) / "123"
            process.mkdir()
            # Fields 3..24, with PPID=42 and RSS=7 pages.
            fields = ["S", "42"] + ["0"] * 19 + ["7"]
            (process / "stat").write_text("123 (name with ) parentheses) " + " ".join(fields))
            snapshot = BENCH["process_snapshot"](Path(root))
            self.assertEqual(snapshot[123], (42, 7 * BENCH["os"].sysconf("SC_PAGE_SIZE")))


if __name__ == "__main__":
    unittest.main()
