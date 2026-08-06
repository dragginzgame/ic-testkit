# AGENTS.md

Repository-specific instructions for agents working on `ic-testkit`.

1. Never update the workspace `Cargo.toml` `workspace.package.version` for `ic-testkit` itself. Version bumps are handled manually by the maintainer.
2. Prefer keeping this crate generic over adding application-specific test harness behavior.
3. Run only targeted tests relevant to the files and behavior changed. Do not run the broad `make test` or `make release-check` gates; the maintainer runs full pre-push validation.
