# Browser fixture sessions

The private `@dragginzgame/ic-testkit-browser` Node package is consumed from an
immutable Git commit. It has no npm dependencies or installation scripts, and is
not part of the published Rust crate. Node 22 or later is required.

`withFixtureSession(options, async connection => { ... })` spawns one fixture,
removes stale readiness data, waits for complete JSON and passes it to the body.
Options select `command`, `args`, `cwd`, `env`, `readyPath`, `logPath`, and explicit
`startupMs`, `stopMs`, `pollMs` integers in 1..2147483647. The caller creates parent
directories and owns executable/environment admission, readiness schema, source
custody, body deadlines and the browser lifecycle. Paths must be distinct,
caller-owned files. Logs append, including on startup/body failure.

Cleanup sends a newline, then SIGTERM and SIGKILL, waiting at most `stopMs` after
each stage. It reaps the direct child; the fixture must manage its descendants.
It preserves body failures, including falsy exceptions, and aggregates cleanup
failures. This helper targets Linux/macOS, not Windows process termination.

Run `node --test testing/browser/session.test.mjs`. Tests use disposable Node
children; installed PocketIC/browser qualification remains with the consumer.
