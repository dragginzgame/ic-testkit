# AGENTS.md

Repository-specific instructions for agents working on `ic-testkit`.

1. Never update the workspace `Cargo.toml` `workspace.package.version` for `ic-testkit` itself. Version bumps are handled manually by the maintainer.
2. Prefer keeping this crate generic over adding application-specific test harness behavior.
3. Run only targeted tests relevant to the files and behavior changed. Do not run the broad `make test` or `make release-check` gates; the maintainer runs full pre-push validation.
4. Before `1.0`, make API, schema, and behavior changes as hard cuts. Do not add backwards-compatibility shims, aliases, deprecated bridges, dual old/new entry points, or anti-resurrection tests for removed APIs.
5. Keep every repository-owned cache, stamp, schema, protocol, digest-domain, and other format identifier at `v1` before `1.0`; change its `v1` semantics in place instead of introducing `v2` migrations or readers for older formats.
