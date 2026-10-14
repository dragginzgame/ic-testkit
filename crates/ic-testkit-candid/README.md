# ic-testkit-candid

A small native/Wasm Candid reply decoder with only Candid and Serde dependencies.
It has no PocketIC, process, transport, installation or implicit network behavior.

Callers supply `DecodeLimits` for original wire bytes, decoding/skipping work,
type-table entries and header bytes. There is no universal default or implicit
quota calculation. Preserve caller-selected proportional work policy when moving
from an existing decoder. Callers retain endpoints, trust, deadlines, input quality,
raw evidence publication and application error mapping.

`decode_one` retains Candid's single requested result with bounded skipping of
additional arguments. `decode_exactly_one` rejects additional arguments and
trailing bytes. These are explicit current policies, not compatibility fallbacks.
Both return typed wire admission or original Candid failures without modifying
input bytes. This crate does not change Testkit's existing PocketIC call defaults.

The workspace owns this package's version and Rust 1.88 floor. Its separate
publication is required before consumers adopt it from crates.io; adding a
workspace member does not publish it or change consumer dependency graphs.
