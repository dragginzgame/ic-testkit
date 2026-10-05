# Host support and qualification

The required native development and operator hosts are Ubuntu 24.04 x86-64
and macOS 15 on Apple Silicon (ARM64) and Intel (x86-64). The crate's canister
runtime target remains `wasm32-unknown-unknown`. Windows host workflows are
not qualified.

| Host | CI runner | Native qualification |
| --- | --- | --- |
| Ubuntu 24.04 x86-64 | `ubuntu-24.04` | Portable tooling and PocketIC ownership tests |
| macOS 15 ARM64 | `macos-15` | Portable tooling and PocketIC ownership tests |
| macOS 15 x86-64 | `macos-15-intel` | Portable tooling and PocketIC ownership tests |

Runner labels and architectures follow the [GitHub runner inventory](https://github.com/actions/runner-images#available-images).
The matrix declares required support; each revision needs a successful matching
native workflow run. Linux checks do not qualify macOS behavior. The complete
CI, MSRV, archive verification and publish dry-run gates are configured on all
three native hosts. Native macOS qualification remains pending until matching
workflow runs pass. Do not publish or push
just to qualify tooling.

## Prerequisites

- Bash 3.2 or newer. On macOS, use `/bin/bash` to exercise the system shell.
- GNU Make (the macOS system Make is sufficient), Git, Perl, and standard Unix
  tools including `awk`, `cat`, `find`, `grep`, `mktemp`, `sed`, `sort` and `tar`.
- A SHA-256 implementation: Linux `sha256sum` or macOS `shasum -a 256`.
- Rustup with the repository's pinned toolchain, `rustfmt`, `clippy`, and the
  `wasm32-unknown-unknown` target. Published MSRV checks use Rust 1.88.
- `cloc` and `jq` for `make cloc`. These are optional for builds; install them
  using the host's package manager before requesting a LOC report.
- Live PocketIC tests require a compatible native PocketIC 16 binary. Set
  `POCKET_IC_BIN` to its exact local path to avoid upstream automatic download.
  Managed startup takes an explicit caller-provided binary and owns its child.
- Publishing additionally requires Cargo registry credentials. Mocked release
  and publish guard checks do not use credentials or perform remote writes.

Release CI runs `make ci` with the caller's temporary-directory environment.
It preserves Cargo artifacts and does not remove temporary files underneath
upstream background servers. Managed server handles own their process cleanup.

## Focused qualification

Run these commands sequentially, with no other validation modifying their
inputs. The shell guards use fixtures and mock external commands:

```bash
/bin/bash scripts/ci/verify-shared-tooling-snapshot.sh
/bin/bash scripts/ci/check-release-guards.sh
/bin/bash scripts/ci/check-publish-guards.sh
/bin/bash scripts/ci/check-github-actions-pinned.sh
cargo fetch --locked
cargo test -p ic-testkit --locked --offline --lib pic::startup::tests
cargo test -p ic-testkit --locked --offline --test pocket_ic_concurrency
```

Fetching is an explicit preparation step and preserves the selected lockfile.
For an already prepared offline host, use `cargo fetch --locked --offline`;
missing cache data is a preparation failure, not permission to select upgrades.
The focused startup tests use synthetic servers; the caller-provided live
server test is ignored unless explicitly selected as described in the README.

The `portable-hosts` and `pocket-ic-concurrency` workflow jobs exercise the
declared native hosts. For LOC tooling after installing its prerequisites:

```bash
/bin/bash scripts/dev/cloc.sh
```

The maintainer owns full pre-push, release and publication gates. Agents run
only checks affected by their authorized changes.
