# AGENTS.md

Repository-specific instructions for agents working on `ic-testkit`.

1. Never update the `Cargo.toml` package version for `ic-testkit` itself. Version bumps are handled manually by the maintainer.
2. Prefer keeping this crate generic over adding application-specific test harness behavior.
3. Before release-oriented changes, run `make release-check` when network access is available, or at minimum run `make test` and explain any skipped checks.
