#![cfg(not(target_arch = "wasm32"))]

use ic_testkit_native::{AlreadyMeasuring, MeasuredAllocator, measure};
use std::{alloc::System, hint::black_box};

#[global_allocator]
static ALLOCATOR: MeasuredAllocator = MeasuredAllocator::new(System);

#[test]
fn allocations_reallocation_and_preexisting_frees_are_accounted() {
    let counts = measure(3, || {
        let mut bytes = Vec::<u8>::with_capacity(8);
        bytes.push(black_box(1));
        bytes.reserve_exact(8);
        black_box(&bytes);
    })
    .unwrap();
    assert_eq!(counts.allocations, 3);
    assert_eq!(counts.reallocations, 3);
    assert_eq!(counts.net_bytes, 0);
    assert_eq!(counts.peak_bytes, 9);
    let old = Box::new(black_box([0_u8; 16]));
    let mut old = Some(old);
    let freed = measure(1, || drop(old.take())).unwrap();
    assert_eq!(freed.net_bytes, -16);
    assert_eq!(freed.peak_bytes, 0);
}

#[test]
fn nesting_refuses_and_unwinding_restores_the_scope() {
    measure(1, || {
        assert_eq!(
            measure(1, || panic!("nested workload must not run")),
            Err(AlreadyMeasuring)
        );
    })
    .unwrap();
    assert!(std::panic::catch_unwind(|| measure(1, || panic!("workload failed"))).is_err());
    let counts = measure(1, || drop(black_box(Box::new([1_u8; 32])))).unwrap();
    assert_eq!(counts.allocations, 1);
    assert_eq!(counts.net_bytes, 0);
    assert_eq!(counts.peak_bytes, 32);
}
