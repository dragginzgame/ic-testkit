use super::*;
use std::mem::MaybeUninit;

#[test]
fn clones_share_bytes_counts_and_reset_without_counting_observation() {
    let memory = MeteredMemory::<VectorMemory>::default();
    let alias = memory.clone();
    assert_eq!(memory.grow(1), 0);
    memory.write(0, b"abc");
    let mut bytes = [0; 3];
    alias.read(0, &mut bytes);
    assert_eq!(&bytes, b"abc");
    assert_eq!(alias.backing().size(), 1);
    assert_eq!(
        alias.counts(),
        MemoryIoCounts {
            reads: 1,
            read_bytes: 3,
            writes: 1,
            write_bytes: 3,
            grows: 1,
            grown_pages: 1,
            ..MemoryIoCounts::default()
        }
    );
    alias.reset();
    assert_eq!(memory.counts(), MemoryIoCounts::default());
    memory.read(0, &mut bytes);
    assert_eq!(&bytes, b"abc");
}

struct UnsafeOnly;
impl Memory for UnsafeOnly {
    fn size(&self) -> u64 {
        1
    }
    fn grow(&self, pages: u64) -> i64 {
        if pages == 0 { 1 } else { -1 }
    }
    fn read(&self, _: u64, _: &mut [u8]) {
        panic!("safe fallback must not run")
    }
    unsafe fn read_unsafe(&self, _: u64, dst: *mut u8, count: usize) {
        // SAFETY: Caller supplies count valid writable bytes.
        unsafe { dst.write_bytes(42, count) };
    }
    fn write(&self, _: u64, _: &[u8]) {
        panic!("injected write failure")
    }
}

#[test]
fn specialized_unsafe_reads_and_refused_growth_preserve_backing_contracts() {
    let memory = MeteredMemory::new(UnsafeOnly);
    let mut bytes = MaybeUninit::<[u8; 3]>::uninit();
    // SAFETY: The local destination has three writable bytes disjoint from memory.
    unsafe { memory.read_unsafe(0, bytes.as_mut_ptr().cast(), 3) };
    // SAFETY: The successful specialized read initialized all three bytes.
    assert_eq!(unsafe { bytes.assume_init() }, [42; 3]);
    assert_eq!(memory.grow(0), 1);
    assert_eq!(memory.grow(9), -1);
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| memory.write(0, b"abc")));
    assert!(result.is_err());
    assert_eq!(
        memory.counts(),
        MemoryIoCounts {
            reads: 1,
            read_bytes: 3,
            unsafe_reads: 1,
            unsafe_read_bytes: 3,
            writes: 1,
            write_bytes: 3,
            grows: 2,
            ..MemoryIoCounts::default()
        }
    );
}
