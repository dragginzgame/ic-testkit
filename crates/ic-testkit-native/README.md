# IC Testkit Native

Default-feature native allocation measurement has no dependencies and exposes
no APIs on wasm32. This package has no PocketIC or Host dependencies. The
optional `stable-memory` feature adds only `ic-stable-structures` and exposes
portable memory IO fixtures, including for wasm32 test compilation.

Install `MeasuredAllocator::new(System)` as the test binary's global allocator,
or wrap another allocator. `measure(iterations, operation)` returns allocation
and reallocation counts, net bytes, peak positive net bytes and elapsed time for
this thread only. Keep workload construction and output formatting outside the
measurement. Other threads are deliberately excluded; this is not a process RSS
or IC instruction measurement.

Allocations freed during the scope may predate it: net bytes are signed and the
peak is relative to the scope's starting point. Nested measurement refuses before
running its operation. Unwinding disables measurement; the original panic remains.

With `stable-memory`, `memory::MeteredMemory::new(backing)` wraps any selected
`Memory`. Clones share backing and counters; `reset()` starts another observation
window without clearing memory. `counts()` reports read/write attempts and
completed growth calls, counting requested growth pages only on success.
Unsafe reads preserve specialized backing IO and have separate counters as well
as contributing to read totals. Direct `backing()` inspection does not count IO.
Counters saturate; workloads and refusal injection remain with the caller.
