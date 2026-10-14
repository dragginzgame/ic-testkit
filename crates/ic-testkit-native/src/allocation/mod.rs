//! Thread-local native allocation measurements without runtime dependencies.

use std::{
    alloc::{GlobalAlloc, Layout, System},
    cell::Cell,
    time::{Duration, Instant},
};

///
/// Measurement
///
/// Counters for one thread's scoped workload, excluding setup and reporting.
///

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Measurement {
    /// Successful allocation calls, excluding reallocations.
    pub allocations: usize,
    /// Successful reallocation calls.
    pub reallocations: usize,
    /// Allocated minus freed bytes; may be negative for preexisting allocations.
    pub net_bytes: i128,
    /// Peak positive net bytes relative to scope entry, not process heap or RSS.
    pub peak_bytes: usize,
    /// Native elapsed time for the workload.
    pub elapsed: Duration,
}

#[derive(Clone, Copy, Default)]
struct State {
    active: bool,
    counts: Measurement,
}
thread_local! { static STATE: Cell<State> = const { Cell::new(State {
    active: false,
    counts: Measurement { allocations: 0, reallocations: 0, net_bytes: 0, peak_bytes: 0, elapsed: Duration::ZERO },
}) }; }

fn account(old: usize, new: usize, allocation: bool, reallocation: bool) {
    let _ = STATE.try_with(|cell| {
        let mut state = cell.get();
        if state.active {
            state.counts.net_bytes += new as i128 - old as i128;
            state.counts.peak_bytes = state
                .counts
                .peak_bytes
                .max(usize::try_from(state.counts.net_bytes).unwrap_or(0));
            state.counts.allocations += usize::from(allocation);
            state.counts.reallocations += usize::from(reallocation);
            cell.set(state);
        }
    });
}

///
/// MeasuredAllocator
///
/// Instrument an existing global allocator while preserving its allocation contract.
/// Counters are opt-in and scoped to the calling thread.
///

pub struct MeasuredAllocator<A = System> {
    allocator: A,
}
impl<A> MeasuredAllocator<A> {
    /// Wrap the allocator selected by this test binary.
    #[must_use]
    pub const fn new(allocator: A) -> Self {
        Self { allocator }
    }
}

#[expect(
    unsafe_code,
    reason = "forward the GlobalAlloc contract to the caller-selected allocator"
)]
unsafe impl<A: GlobalAlloc> GlobalAlloc for MeasuredAllocator<A> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // SAFETY: The caller supplies GlobalAlloc's valid layout unchanged.
        let pointer = unsafe { self.allocator.alloc(layout) };
        if !pointer.is_null() {
            account(0, layout.size(), true, false);
        }
        pointer
    }
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        // SAFETY: Forward the valid layout and preserve zero-initialization.
        let pointer = unsafe { self.allocator.alloc_zeroed(layout) };
        if !pointer.is_null() {
            account(0, layout.size(), true, false);
        }
        pointer
    }
    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        account(layout.size(), 0, false, false);
        // SAFETY: Forward the caller's allocation and matching layout unchanged.
        unsafe { self.allocator.dealloc(pointer, layout) };
    }
    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        // SAFETY: Forward the valid pointer/layout and requested nonzero size.
        let result = unsafe { self.allocator.realloc(pointer, layout, size) };
        if !result.is_null() {
            account(layout.size(), size, false, true);
        }
        result
    }
}

///
/// AlreadyMeasuring
///
/// A nested measurement was refused without executing its workload.
///

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AlreadyMeasuring;
impl std::fmt::Display for AlreadyMeasuring {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("allocation measurement already active on this thread")
    }
}
impl std::error::Error for AlreadyMeasuring {}

struct Guard;
impl Drop for Guard {
    fn drop(&mut self) {
        STATE.with(|cell| cell.set(State::default()));
    }
}

/// Measure a workload on this thread; nested scopes refuse before executing.
///
/// Setup, result formatting and other threads are excluded. Panics propagate,
/// and an unwind guard disables accounting before returning control to the caller.
///
/// # Errors
/// Returns `AlreadyMeasuring` when called from an active measurement.
pub fn measure(
    iterations: usize,
    mut operation: impl FnMut(),
) -> Result<Measurement, AlreadyMeasuring> {
    STATE.with(|cell| {
        if cell.get().active {
            return Err(AlreadyMeasuring);
        }
        cell.set(State {
            active: true,
            ..State::default()
        });
        Ok(())
    })?;
    let guard = Guard;
    let started = Instant::now();
    for _ in 0..iterations {
        operation();
    }
    let elapsed = started.elapsed();
    let mut counts = STATE.with(|cell| cell.get().counts);
    drop(guard);
    counts.elapsed = elapsed;
    Ok(counts)
}
