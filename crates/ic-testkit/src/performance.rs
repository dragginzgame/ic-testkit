//! Canister-side benchmark marker emission using IC execution counters.

use crate::benchmark::{BenchmarkCounters, DEFAULT_PREFIX, format_marker};

const WASM_PAGE_BYTES: u128 = 65_536;

/// Capture and emit compact benchmark counter markers.
pub struct Performance;

impl Performance {
    /// Print one `ICTK` marker for `label` using the current counters.
    ///
    /// Pair labels ending in `:start` and `:end` for host-side span analysis.
    pub fn measure(label: &str) {
        ic_cdk::api::debug_print(format_marker(DEFAULT_PREFIX, label, Self::counters()));
    }

    /// Read the instruction, Wasm heap, stable-memory, and allocation counters.
    ///
    /// `total_allocation` is currently reserved and emitted as zero.
    #[must_use]
    pub fn counters() -> BenchmarkCounters {
        BenchmarkCounters {
            instructions: u128::from(ic_cdk::api::call_context_instruction_counter()),
            heap_bytes: wasm_memory_size_bytes(),
            memory_bytes: u128::from(ic_cdk::api::stable_size()) * WASM_PAGE_BYTES,
            total_allocation: 0,
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn wasm_memory_size_bytes() -> u128 {
    u128::try_from(core::arch::wasm32::memory_size(0)).expect("usize fits into u128")
        * WASM_PAGE_BYTES
}

#[cfg(not(target_arch = "wasm32"))]
const fn wasm_memory_size_bytes() -> u128 {
    0
}
