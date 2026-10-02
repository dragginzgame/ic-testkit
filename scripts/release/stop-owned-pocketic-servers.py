#!/usr/bin/env python3
"""Stop PocketIC servers using this release invocation's private port files."""

import os
from pathlib import Path
import select
import signal
import sys
import time


def executable_identity(path):
    metadata = path.stat()
    return metadata.st_dev, metadata.st_ino


def owned_server(process, scratch, server_identities):
    try:
        arguments = (process / "cmdline").read_bytes().split(b"\0")
    except (FileNotFoundError, ProcessLookupError):
        return False
    for index, argument in enumerate(arguments[1:-1], start=1):
        if argument == b"--port-file":
            port = Path(os.fsdecode(arguments[index + 1]))
            if port.is_absolute() and scratch in port.parents:
                # Reject paths that escape through '..' or a symlink.
                if scratch not in port.resolve().parents:
                    continue
                try:
                    identity = executable_identity(process / "exe")
                except (FileNotFoundError, ProcessLookupError):
                    # The process may have exited between reading argv and exe.
                    if not (process / "cmdline").read_bytes():
                        return False
                    raise RuntimeError(f"cannot identify executable for process {process.name}")
                if identity not in server_identities:
                    raise RuntimeError(
                        f"unknown executable for process {process.name} using a private port file"
                    )
                return True
    return False


def wait_for_exit(handles, timeout):
    if not handles:
        return []
    # A pidfd becomes readable when its exact process exits, including zombies.
    # select may wake for just one process, so share a deadline across the set.
    deadline = time.monotonic() + timeout
    pending = list(handles)
    while pending:
        ready, _, _ = select.select(pending, [], [], max(0, deadline - time.monotonic()))
        if not ready:
            break
        pending = [handle for handle in pending if handle not in ready]
    return pending


def stop_servers(scratch, server_binaries, timeout=5):
    if scratch.is_symlink() or not scratch.is_dir():
        raise ValueError("release scratch must be an existing regular directory")
    scratch = scratch.resolve()
    if not scratch.name.startswith("ic-testkit-release-ci."):
        raise ValueError("unexpected release scratch directory name")
    if not Path("/proc/self/cmdline").exists():
        raise RuntimeError("safe release server cleanup requires Linux /proc")
    if not hasattr(os, "pidfd_open") or not hasattr(signal, "pidfd_send_signal"):
        raise RuntimeError("safe release server cleanup requires Python pidfd support")

    # Match /proc/PID/exe by device/inode, rather than trusting argv[0]. Missing
    # binaries are possible when CI never started a server. A live process using
    # a private port file must still match an existing selected binary.
    server_identities = set()
    for binary in server_binaries:
        if not binary.is_absolute():
            raise ValueError("selected server binary paths must be absolute")
        try:
            server_identities.add(executable_identity(binary))
        except FileNotFoundError:
            pass

    handles = []
    try:
        for process in Path("/proc").iterdir():
            if not process.name.isdecimal():
                continue
            try:
                # Ignore other users, but fail closed on inaccessible processes
                # owned by the release runner's user.
                if process.stat().st_uid != os.getuid():
                    continue
                if not owned_server(process, scratch, server_identities):
                    continue
                handle = os.pidfd_open(int(process.name))
            except (ProcessLookupError, FileNotFoundError):
                continue
            handles.append(handle)
            # Check ownership after opening the pidfd. Signals use the captured
            # process identity, never a numeric PID that could have been reused.
            if not owned_server(process, scratch, server_identities):
                handles.remove(handle)
                os.close(handle)
                continue

        if handles:
            print(f"Stopping {len(handles)} release-owned PocketIC server(s).", flush=True)
        for handle in handles:
            try:
                signal.pidfd_send_signal(handle, signal.SIGTERM)
            except ProcessLookupError:
                pass
        pending = wait_for_exit(handles, timeout)
        for handle in pending:
            try:
                signal.pidfd_send_signal(handle, signal.SIGKILL)
            except ProcessLookupError:
                pass
        if wait_for_exit(pending, timeout):
            raise RuntimeError("a release-owned PocketIC server did not exit")
        # Refuse directory deletion if another server appeared during cleanup.
        for process in Path("/proc").iterdir():
            if process.name.isdecimal():
                try:
                    if process.stat().st_uid != os.getuid():
                        continue
                    if owned_server(process, scratch, server_identities):
                        raise RuntimeError("a release-owned PocketIC server appeared during cleanup")
                except (ProcessLookupError, FileNotFoundError):
                    continue
    finally:
        for handle in handles:
            os.close(handle)


if __name__ == "__main__":
    try:
        if len(sys.argv) < 3:
            raise ValueError(
                "usage: stop-owned-pocketic-servers.py RELEASE_SCRATCH SERVER_BINARY..."
            )
        stop_servers(Path(sys.argv[1]), [Path(binary) for binary in sys.argv[2:]])
    except (OSError, ValueError, RuntimeError) as error:
        print(f"PocketIC cleanup failed: {error}", file=sys.stderr)
        sys.exit(1)
