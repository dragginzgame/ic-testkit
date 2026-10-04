#!/usr/bin/env python3
"""Focused measurement-boundary checks; no PocketIC or Cargo work."""

from pathlib import Path
import runpy
import tempfile
import unittest

BENCH = runpy.run_path(str(Path(__file__).with_name("benchmark-fixture-reuse.py")))


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
