#!/usr/bin/env python3
"""Targeted process/socket regressions for release scratch cleanup."""

import importlib.util
import os
from pathlib import Path
import signal
import shutil
import subprocess
import sys
import tempfile
import time
import unittest
from unittest import mock


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
        self.bin_dir = Path(self.temp.name) / "bin"
        self.bin_dir.mkdir()
        self.binary = self.bin_dir / "pocket-ic"
        # Use distinct native executables so /proc/PID/exe tests real identity,
        # including when argv[0] claims to be a different binary.
        shutil.copy2(sys.executable, self.binary)

    def spawn(
        self, scratch, name="pocket_ic_123.port", label="pocket-ic",
        ignore_term=False, argv0=None,
    ):
        binary = self.bin_dir / label
        if not binary.exists():
            shutil.copy2(sys.executable, binary)
        port = scratch / name
        marker = self.root / (name + ".stopped")
        env = dict(os.environ, STOP_MARKER=str(marker))
        if ignore_term:
            env["IGNORE_TERM"] = "1"
        process = subprocess.Popen(
            [str(argv0 or binary), "-c", PEER, "--port-file", str(port)],
            executable=str(binary),
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
        cleanup.stop_servers(self.root, [self.binary])
        self.assertEqual(process.wait(timeout=5), 0)
        self.assertEqual(marker.read_text(), "True")

    def test_does_not_stop_an_external_server(self):
        external = self.root / "external"
        external.mkdir()
        scratch = self.root / "ic-testkit-release-ci.owned"
        scratch.mkdir()
        outside, _ = self.spawn(external, "pocket_ic_456.port")
        owned, _ = self.spawn(scratch, "pocket_ic_789.port")
        cleanup.stop_servers(scratch, [self.binary])
        self.assertEqual(owned.wait(timeout=5), 0)
        self.assertIsNone(outside.poll())

    def test_unknown_executable_fails_without_signalling_an_unrelated_process(self):
        # Even an exact selected argv[0] must not override the kernel identity.
        process, _ = self.spawn(self.root, label="unrelated-peer", argv0=self.binary)
        with self.assertRaisesRegex(RuntimeError, "unknown executable"):
            cleanup.stop_servers(self.root, [self.binary])
        self.assertIsNone(process.poll())

    def test_stops_a_selected_renamed_binary(self):
        process, marker = self.spawn(self.root, label="pocket-ic-v16")
        cleanup.stop_servers(self.root, [self.bin_dir / "pocket-ic-v16"])
        self.assertEqual(process.wait(timeout=5), 0)
        self.assertEqual(marker.read_text(), "True")

    def test_matches_a_selected_binary_through_a_symlink(self):
        alias = self.bin_dir / "selected-server"
        alias.symlink_to(self.binary)
        process, _ = self.spawn(self.root)
        cleanup.stop_servers(self.root, [alias])
        self.assertEqual(process.wait(timeout=5), 0)

    def test_missing_selected_binary_fails_closed_for_a_live_server(self):
        process, _ = self.spawn(self.root)
        self.binary.unlink()
        with self.assertRaisesRegex(RuntimeError, "unknown executable"):
            cleanup.stop_servers(self.root, [self.binary])
        self.assertIsNone(process.poll())

    def test_replaced_selected_binary_fails_closed_for_a_live_server(self):
        process, _ = self.spawn(self.root)
        replacement = self.bin_dir / "replacement"
        shutil.copy2(sys.executable, replacement)
        replacement.replace(self.binary)
        with self.assertRaisesRegex(RuntimeError, "unknown executable"):
            cleanup.stop_servers(self.root, [self.binary])
        self.assertIsNone(process.poll())

    def test_inaccessible_executable_fails_closed_for_a_live_server(self):
        process, _ = self.spawn(self.root)
        identity = cleanup.executable_identity

        def inaccessible(candidate):
            if candidate == Path(f"/proc/{process.pid}/exe"):
                raise PermissionError("executable identity unavailable")
            return identity(candidate)

        with mock.patch.object(cleanup, "executable_identity", side_effect=inaccessible):
            with self.assertRaises(PermissionError):
                cleanup.stop_servers(self.root, [self.binary])
        self.assertIsNone(process.poll())

    def test_rechecks_ownership_after_opening_the_pidfd(self):
        process, _ = self.spawn(self.root)
        check = cleanup.owned_server
        checks = 0

        def changed_ownership(candidate, *args):
            nonlocal checks
            if candidate.name != str(process.pid):
                return check(candidate, *args)
            checks += 1
            # Simulate losing the private port-file argument after discovery.
            return check(candidate, *args) if checks == 1 else False

        with mock.patch.object(cleanup, "owned_server", side_effect=changed_ownership):
            cleanup.stop_servers(self.root, [self.binary])
        self.assertEqual(checks, 3)
        self.assertIsNone(process.poll())

    def test_final_scan_rejects_a_server_that_appears_during_cleanup(self):
        process, _ = self.spawn(self.root)
        wait = cleanup.wait_for_exit
        late = None

        def start_late_server(handles, timeout):
            nonlocal late
            pending = wait(handles, timeout)
            if late is None:
                late, _ = self.spawn(self.root, "pocket_ic_late.port")
            return pending

        with mock.patch.object(cleanup, "wait_for_exit", side_effect=start_late_server):
            with self.assertRaisesRegex(RuntimeError, "appeared during cleanup"):
                cleanup.stop_servers(self.root, [self.binary])
        self.assertEqual(process.wait(timeout=5), 0)
        self.assertIsNone(late.poll())

    def test_kills_an_owned_server_that_ignores_termination(self):
        process, _ = self.spawn(self.root, ignore_term=True)
        cleanup.stop_servers(self.root, [self.binary], timeout=0.1)
        self.assertEqual(process.wait(timeout=5), -signal.SIGKILL)

    def test_rejects_symlink_scratch_without_signalling_its_server(self):
        process, _ = self.spawn(self.root)
        alias = self.root.parent / (self.root.name + ".alias")
        alias.symlink_to(self.root, target_is_directory=True)
        self.addCleanup(alias.unlink)
        with self.assertRaises(ValueError):
            cleanup.stop_servers(alias, [self.binary])
        self.assertIsNone(process.poll())

    def test_does_not_claim_an_escaping_port_path(self):
        external = self.root / "external"
        external.mkdir()
        scratch = self.root / "ic-testkit-release-ci.owned"
        scratch.mkdir()
        process, _ = self.spawn(scratch, "../external/pocket_ic_123.port")
        cleanup.stop_servers(scratch, [self.binary])
        self.assertIsNone(process.poll())

    def test_does_not_claim_a_port_path_that_escapes_through_a_symlink(self):
        external = Path(self.temp.name) / "external"
        external.mkdir()
        (self.root / "link").symlink_to(external, target_is_directory=True)
        process, _ = self.spawn(self.root, "link/pocket_ic_123.port")
        cleanup.stop_servers(self.root, [self.binary])
        self.assertIsNone(process.poll())

    def test_release_runner_checks_binary_identity_and_preserves_ci_status(self):
        selections = [
            "default", "POCKET_IC_BIN", "relative", "PATH",
            "IC_TESTKIT_POCKET_IC_SERVER", "unknown",
        ]
        cases = [(selection, status) for selection in selections for status in [0, 23]]
        for selection, status in cases:
            with self.subTest(selection=selection, status=status):
                case = self.root / f"{selection}-{status}"
                case.mkdir()
                trace = case / "scratch"
                marker = case / "stopped"

                binary = case / "pocket-ic-v16"
                shutil.copy2(self.binary, binary)

                def stop_survivors(trace=trace, binary=binary):
                    if trace.exists():
                        scratch = Path(trace.read_text())
                        if scratch.is_dir():
                            cleanup.stop_servers(
                                scratch, [binary, scratch / "pocket-ic-server-16.0.0/pocket-ic"],
                                timeout=0.1,
                            )

                self.addCleanup(stop_survivors)
                make = case / "make"
                make.write_text(
                    f"#!{sys.executable}\n"
                    "import os, pathlib, shutil, subprocess, sys, time\n"
                    "scratch = pathlib.Path(os.environ['TMPDIR'])\n"
                    "port = scratch / 'pocket_ic_123.port'\n"
                    "pathlib.Path(os.environ['SCRATCH_TRACE']).write_text(str(scratch))\n"
                    "binary = pathlib.Path(os.environ['PEER_BINARY'])\n"
                    "if os.environ['SELECTION'] == 'default':\n"
                    "    selected = scratch / 'pocket-ic-server-16.0.0/pocket-ic'\n"
                    "    selected.parent.mkdir()\n"
                    "    shutil.copy2(binary, selected)\n"
                    "    binary = selected\n"
                    f"subprocess.Popen([str(binary), '-c', {PEER!r}, "
                    "'--port-file', str(port)], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)\n"
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
                    PEER_BINARY=str(binary),
                    SELECTION=selection,
                )
                env.pop("POCKET_IC_BIN", None)
                env.pop("IC_TESTKIT_POCKET_IC_SERVER", None)
                if selection == "relative":
                    env["POCKET_IC_BIN"] = os.path.relpath(binary, REPO)
                elif selection == "PATH":
                    env["POCKET_IC_BIN"] = binary.name
                    env["PATH"] = str(case) + os.pathsep + env["PATH"]
                elif selection == "unknown":
                    env["POCKET_IC_BIN"] = str(self.binary)
                elif selection != "default":
                    env[selection] = str(binary)
                result = subprocess.run(
                    ["bash", str(REPO / "scripts/release/run-ci.sh")],
                    env=env,
                    capture_output=True,
                    text=True,
                    timeout=20,
                )
                if selection == "unknown":
                    self.assertEqual(result.returncode, status or 1, result.stderr)
                    scratch = Path(trace.read_text())
                    self.assertTrue((scratch / "pocket_ic_123.sock").exists())
                    self.assertFalse(marker.exists())
                    self.assertIn("unknown executable", result.stderr)
                    self.assertIn("preserving release CI temporary directory", result.stderr)
                else:
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
