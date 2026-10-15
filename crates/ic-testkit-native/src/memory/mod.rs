//! Native backing-memory IO counters; workloads and failure injection stay local.

use ic_stable_structures::{Memory, VectorMemory};
use std::{cell::Cell, rc::Rc};

///
/// MemoryIoCounts
///
/// Safe and unsafe read attempts share read totals; unsafe reads are also
/// counted separately. Writes count attempts before delegation. Growth counts
/// completed calls, with requested pages counted only when growth succeeds.
/// Size queries do not count as IO. Counters saturate at u64::MAX and set a
/// sticky overflow flag on actual overflow. Measurement callers must reject
/// overflowed counts rather than reporting clipped values as exact totals.
///

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct MemoryIoCounts {
    /// Read attempts, including unsafe reads.
    pub reads: u64,
    /// Requested bytes across all read attempts.
    pub read_bytes: u64,
    /// Unsafe read attempts.
    pub unsafe_reads: u64,
    /// Requested bytes across unsafe read attempts.
    pub unsafe_read_bytes: u64,
    /// Write attempts.
    pub writes: u64,
    /// Requested bytes across write attempts.
    pub write_bytes: u64,
    /// Completed growth attempts, including zero-page and refused requests.
    pub grows: u64,
    /// Requested pages of successful growth calls.
    pub grown_pages: u64,
    /// At least one counter overflowed since the last reset.
    pub overflowed: bool,
}

///
/// MeteredMemory
///
/// Shared native memory and IO counters. Clones observe the same backing and
/// counters; resetting any clone resets the whole shared observation window.
/// The wrapper preserves the backing's specialized unsafe read implementation.
/// It does not provide transactions, failure injection or IC instruction counts.
///

pub struct MeteredMemory<M: Memory = VectorMemory> {
    memory: Rc<M>,
    counts: Rc<Cell<MemoryIoCounts>>,
}

impl<M: Memory> MeteredMemory<M> {
    /// Wrap the selected backing with zeroed counters.
    #[must_use]
    pub fn new(memory: M) -> Self {
        Self {
            memory: Rc::new(memory),
            counts: Rc::default(),
        }
    }

    /// Inspect the backing directly without accounting; useful for fixture checks.
    #[must_use]
    pub fn backing(&self) -> &M {
        &self.memory
    }

    /// Clear the counters shared by all clones without changing stored bytes.
    pub fn reset(&self) {
        self.counts.set(MemoryIoCounts::default());
    }

    /// Snapshot the shared counters.
    #[must_use]
    pub fn counts(&self) -> MemoryIoCounts {
        self.counts.get()
    }

    fn record_read(&self, bytes: usize, unsafe_read: bool) {
        let mut counts = self.counts.get();
        add_count(&mut counts.reads, 1, &mut counts.overflowed);
        add_count(&mut counts.read_bytes, bytes as u64, &mut counts.overflowed);
        if unsafe_read {
            add_count(&mut counts.unsafe_reads, 1, &mut counts.overflowed);
            add_count(
                &mut counts.unsafe_read_bytes,
                bytes as u64,
                &mut counts.overflowed,
            );
        }
        self.counts.set(counts);
    }
}

impl<M: Memory> Clone for MeteredMemory<M> {
    fn clone(&self) -> Self {
        Self {
            memory: Rc::clone(&self.memory),
            counts: Rc::clone(&self.counts),
        }
    }
}

impl<M: Memory + Default> Default for MeteredMemory<M> {
    fn default() -> Self {
        Self::new(M::default())
    }
}

impl<M: Memory> Memory for MeteredMemory<M> {
    fn size(&self) -> u64 {
        self.memory.size()
    }

    fn grow(&self, pages: u64) -> i64 {
        let result = self.memory.grow(pages);
        let mut counts = self.counts.get();
        add_count(&mut counts.grows, 1, &mut counts.overflowed);
        if result >= 0 {
            add_count(&mut counts.grown_pages, pages, &mut counts.overflowed);
        }
        self.counts.set(counts);
        result
    }

    fn read(&self, offset: u64, dst: &mut [u8]) {
        self.record_read(dst.len(), false);
        self.memory.read(offset, dst);
    }

    unsafe fn read_unsafe(&self, offset: u64, dst: *mut u8, count: usize) {
        self.record_read(count, true);
        // SAFETY: Forward the caller's valid, nonoverlapping destination unchanged.
        unsafe { self.memory.read_unsafe(offset, dst, count) }
    }

    fn write(&self, offset: u64, src: &[u8]) {
        let mut counts = self.counts.get();
        add_count(&mut counts.writes, 1, &mut counts.overflowed);
        add_count(
            &mut counts.write_bytes,
            src.len() as u64,
            &mut counts.overflowed,
        );
        self.counts.set(counts);
        self.memory.write(offset, src);
    }
}

const fn add_count(value: &mut u64, increment: u64, overflowed: &mut bool) {
    let (sum, overflow) = value.overflowing_add(increment);
    *value = if overflow { u64::MAX } else { sum };
    *overflowed |= overflow;
}

#[cfg(test)]
mod tests;
