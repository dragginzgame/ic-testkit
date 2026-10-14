//! Native allocation measurement and opt-in, target-neutral memory IO fixtures.

#[cfg(not(target_arch = "wasm32"))]
mod allocation;
#[cfg(not(target_arch = "wasm32"))]
pub use allocation::{AlreadyMeasuring, MeasuredAllocator, Measurement, measure};

#[cfg(feature = "stable-memory")]
pub mod memory;
