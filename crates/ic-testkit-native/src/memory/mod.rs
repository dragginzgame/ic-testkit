//! Native backing-memory IO counters; workloads and failure injection stay local.

use ic_stable_structures::{Memory, VectorMemory};
use std::{cell::Cell, rc::Rc};

///
/// MemoryIoCounts
///
/// Safe and unsafe read attempts share read totals; unsafe reads are also
/// counted separately. Writes count attempts before delegation. Growth counts
/// completed calls, with requested pages counted only when growth succeeds.
/// Size queries do not count as IO. Counters saturate at u64::MAX.
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
        counts.reads = counts.reads.saturating_add(1);
        counts.read_bytes = counts.read_bytes.saturating_add(bytes as u64);
        if unsafe_read {
            counts.unsafe_reads = counts.unsafe_reads.saturating_add(1);
            counts.unsafe_read_bytes = counts.unsafe_read_bytes.saturating_add(bytes as u64);
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
        counts.grows = counts.grows.saturating_add(1);
        if result >= 0 {
            counts.grown_pages = counts.grown_pages.saturating_add(pages);
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
        counts.writes = counts.writes.saturating_add(1);
        counts.write_bytes = counts.write_bytes.saturating_add(src.len() as u64);
        self.counts.set(counts);
        self.memory.write(offset, src);
    }
}

#[cfg(test)]
mod tests;
