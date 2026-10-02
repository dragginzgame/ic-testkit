#!/usr/bin/env python3
"""Targeted process/socket regressions for release scratch cleanup."""

import importlib.util
import os
from pathlib import Path
import signal
import subprocess
import sys
import tempfile
import time
import unittest


REPO = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location(
    "cleanup", REPO / "scripts/release/stop-owned-pocketic-servers.py"
)
cleanup = importlib.util.module_from_spec(spec)
spec.loader.exec_module(cleanup)

PEER = r"""
import os, pathlib, signal, socket, sys, time
port = pathlib.Path(sys.argv[-1])
peer = socket.socket(socket.AF_UNIX)
sock = port.with_suffix('.sock')
# Bind relative to the socket's directory: release CI can nest TMPDIR beyond
# AF_UNIX's pathname limit, while the socket must stay in the owned scratch.
os.chdir(sock.parent)
peer.bind(sock.name)
def terminate(signum, frame):
    pathlib.Path(os.environ['STOP_MARKER']).write_text(str(sock.exists()))
    peer.close()
    sock.unlink()
    sys.exit(0)
signal.signal(signal.SIGTERM, signal.SIG_IGN if os.environ.get('IGNORE_TERM') else terminate)
port.write_text(str(os.getpid()))
while True:
    time.sleep(1)
"""


@unittest.skipUnless(
    sys.platform == "linux" and hasattr(signal, "pidfd_send_signal"),
    "release server ownership uses Linux pidfds",
)
class ReleaseCleanupTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="ic-testkit-release-ci.")
        self.addCleanup(self.temp.cleanup)
        # Exercise long scratch paths even when the caller uses plain /tmp.
        self.root = Path(self.temp.name) / ("ic-testkit-release-ci." + "nested-" * 20)
        self.root.mkdir()

    def spawn(self, scratch, name="pocket_ic_123.port", label="pocket-ic", ignore_term=False):
        port = scratch / name
        marker = self.root / (name + ".stopped")
        env = dict(os.environ, STOP_MARKER=str(marker))
        if ignore_term:
            env["IGNORE_TERM"] = "1"
        process = subprocess.Popen(
            [str(scratch / label), "-c", PEER, "--port-file", str(port)],
            executable=sys.executable,
            env=env,
        )

        def stop():
            if process.poll() is None:
                process.kill()
            process.wait(timeout=5)

        self.addCleanup(stop)
        deadline = time.monotonic() + 5
        while not port.exists():
            self.assertIsNone(process.poll(), "socket peer exited before readiness")
            self.assertLess(time.monotonic(), deadline, "socket peer failed to start")
            time.sleep(0.01)
        return process, marker

    def test_stops_owned_server_before_its_socket_directory_is_removed(self):
        process, marker = self.spawn(self.root)
        cleanup.stop_servers(self.root)
        self.assertEqual(process.wait(timeout=5), 0)
        self.assertEqual(marker.read_text(), "True")

    def test_does_not_stop_an_external_server_or_an_unrelated_process(self):
        external = self.root / "external"
        external.mkdir()
        scratch = self.root / "ic-testkit-release-ci.owned"
        scratch.mkdir()
        outside, _ = self.spawn(external, "pocket_ic_456.port")
        unrelated, _ = self.spawn(scratch, label="unrelated-peer")
        owned, _ = self.spawn(scratch, "pocket_ic_789.port")
        cleanup.stop_servers(scratch)
        self.assertEqual(owned.wait(timeout=5), 0)
        self.assertIsNone(outside.poll())
        self.assertIsNone(unrelated.poll())

    def test_kills_an_owned_server_that_ignores_termination(self):
        process, _ = self.spawn(self.root, ignore_term=True)
        cleanup.stop_servers(self.root, timeout=0.1)
        self.assertEqual(process.wait(timeout=5), -signal.SIGKILL)

    def test_rejects_symlink_scratch_without_signalling_its_server(self):
        process, _ = self.spawn(self.root)
        alias = self.root.parent / (self.root.name + ".alias")
        alias.symlink_to(self.root, target_is_directory=True)
        self.addCleanup(alias.unlink)
        with self.assertRaises(ValueError):
            cleanup.stop_servers(alias)
        self.assertIsNone(process.poll())

    def test_does_not_claim_an_escaping_port_path(self):
        external = self.root / "external"
        external.mkdir()
        scratch = self.root / "ic-testkit-release-ci.owned"
        scratch.mkdir()
        process, _ = self.spawn(scratch, "../external/pocket_ic_123.port")
        cleanup.stop_servers(scratch)
        self.assertIsNone(process.poll())

    def test_release_runner_stops_servers_and_preserves_ci_status(self):
        for status in [0, 23]:
            with self.subTest(status=status):
                case = self.root / str(status)
                case.mkdir()
                trace = case / "scratch"
                marker = case / "stopped"

                def stop_survivors(trace=trace):
                    if trace.exists():
                        scratch = Path(trace.read_text())
                        if scratch.is_dir():
                            cleanup.stop_servers(scratch, timeout=0.1)

                self.addCleanup(stop_survivors)
                make = case / "make"
                make.write_text(
                    f"#!{sys.executable}\n"
                    "import os, pathlib, subprocess, sys, time\n"
                    "scratch = pathlib.Path(os.environ['TMPDIR'])\n"
                    "port = scratch / 'pocket_ic_123.port'\n"
                    "pathlib.Path(os.environ['SCRATCH_TRACE']).write_text(str(scratch))\n"
                    f"subprocess.Popen([str(scratch / 'pocket-ic'), '-c', {PEER!r}, "
                    "'--port-file', str(port)], executable=sys.executable)\n"
                    "deadline = time.monotonic() + 5\n"
                    "while not port.exists():\n"
                    "    if time.monotonic() >= deadline: sys.exit(97)\n"
                    "    time.sleep(0.01)\n"
                    "sys.exit(int(os.environ['CI_STATUS']))\n"
                )
                make.chmod(0o700)
                env = dict(
                    os.environ,
                    MAKE=str(make),
                    TMPDIR=str(case),
                    SCRATCH_TRACE=str(trace),
                    STOP_MARKER=str(marker),
                    CI_STATUS=str(status),
                )
                result = subprocess.run(
                    ["bash", str(REPO / "scripts/release/run-ci.sh")],
                    env=env,
                    capture_output=True,
                    text=True,
                    timeout=20,
                )
                self.assertEqual(result.returncode, status, result.stderr)
                self.assertEqual(marker.read_text(), "True")
                self.assertFalse(Path(trace.read_text()).exists())

    def test_release_runner_preserves_scratch_after_cleanup_failure(self):
        for status, expected in [(0, 1), (23, 23)]:
            with self.subTest(status=status):
                case = self.root / str(status)
                case.mkdir()
                bin_dir = case / "bin"
                bin_dir.mkdir()
                trace = case / "scratch"
                make = case / "make"
                make.write_text(
                    "#!/bin/bash\n"
                    'printf "%s" "$TMPDIR" > "$SCRATCH_TRACE"\n'
                    'touch "$TMPDIR/retained-evidence"\n'
                    'exit "$CI_STATUS"\n'
                )
                make.chmod(0o700)
                python = bin_dir / "python3"
                python.write_text("#!/bin/bash\nexit 17\n")
                python.chmod(0o700)
                env = dict(
                    os.environ,
                    MAKE=str(make),
                    TMPDIR=str(case),
                    SCRATCH_TRACE=str(trace),
                    CI_STATUS=str(status),
                    PATH=str(bin_dir) + os.pathsep + os.environ["PATH"],
                )
                result = subprocess.run(
                    ["bash", str(REPO / "scripts/release/run-ci.sh")],
                    env=env,
                    capture_output=True,
                    text=True,
                    timeout=10,
                )
                self.assertEqual(result.returncode, expected, result.stderr)
                scratch = Path(trace.read_text())
                self.assertTrue((scratch / "retained-evidence").exists())
                self.assertIn("preserving release CI temporary directory", result.stderr)


if __name__ == "__main__":
    unittest.main()
