# ic-testkit-candid

A small native/Wasm Candid reply decoder with Candid and Serde as its two direct dependencies.
It has no PocketIC, process, transport, installation or implicit network behavior.

Callers supply `DecodeLimits` for original wire bytes, decoding/skipping work,
type-table entries and header bytes. There is no universal default or implicit
quota calculation. Preserve caller-selected proportional work policy when moving
from an existing decoder. Callers retain endpoints, trust, deadlines, input quality,
raw evidence publication and application error mapping.

`decode_one` retains Candid's single requested result with bounded skipping of
additional arguments. `decode_exactly_one` requires one actual wire argument and
rejects additional arguments and trailing bytes.
`decode_args` decodes top-level argument lists with the same bounded skipping
policy. All three return typed wire admission or original Candid failures without
modifying input bytes. Testkit's PocketIC helpers require these same explicit
limits and preserve `DecodeError` as the source of `CandidCallError::Decode`.

The workspace owns this package's version and Rust 1.91 floor. The workspace
publication adapter observes/publishes this package before Testkit, preserving
partial-publication retry admission. Reviewed publication is required before
consumers adopt it; adding a workspace member does not change their graphs.
