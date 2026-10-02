#!/usr/bin/env python3
"""Stop PocketIC servers using this release invocation's private port files."""

import os
from pathlib import Path
import select
import signal
import sys
import time


def owned_server(process, scratch):
    try:
        arguments = (process / "cmdline").read_bytes().split(b"\0")
    except (FileNotFoundError, ProcessLookupError):
        return False
    if not arguments or Path(os.fsdecode(arguments[0])).name != "pocket-ic":
        return False
    for index, argument in enumerate(arguments[:-1]):
        if argument == b"--port-file":
            port = Path(os.fsdecode(arguments[index + 1]))
            if port.is_absolute() and scratch in port.parents:
                # Reject paths that escape through '..' or a symlink.
                return scratch in port.resolve().parents
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


def stop_servers(scratch, timeout=5):
    if scratch.is_symlink() or not scratch.is_dir():
        raise ValueError("release scratch must be an existing regular directory")
    scratch = scratch.resolve()
    if not scratch.name.startswith("ic-testkit-release-ci."):
        raise ValueError("unexpected release scratch directory name")
    if not Path("/proc/self/cmdline").exists():
        raise RuntimeError("safe release server cleanup requires Linux /proc")
    if not hasattr(os, "pidfd_open") or not hasattr(signal, "pidfd_send_signal"):
        raise RuntimeError("safe release server cleanup requires Python pidfd support")

    handles = []
    try:
        for process in Path("/proc").iterdir():
            if not process.name.isdecimal():
                continue
            try:
                if not owned_server(process, scratch):
                    continue
                handle = os.pidfd_open(int(process.name))
            except (ProcessLookupError, FileNotFoundError):
                continue
            except PermissionError:
                # Other users' processes cannot belong to this invocation.
                continue
            handles.append(handle)
            # Check ownership after opening the pidfd. Signals use the captured
            # process identity, never a numeric PID that could have been reused.
            if not owned_server(process, scratch):
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
                    if owned_server(process, scratch):
                        raise RuntimeError("a release-owned PocketIC server appeared during cleanup")
                except PermissionError:
                    continue
    finally:
        for handle in handles:
            os.close(handle)


if __name__ == "__main__":
    try:
        if len(sys.argv) != 2:
            raise ValueError("usage: stop-owned-pocketic-servers.py RELEASE_SCRATCH")
        stop_servers(Path(sys.argv[1]))
    except (OSError, ValueError, RuntimeError) as error:
        print(f"PocketIC cleanup failed: {error}", file=sys.stderr)
        sys.exit(1)
