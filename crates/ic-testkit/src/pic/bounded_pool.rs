use std::{
    collections::VecDeque,
    num::NonZeroUsize,
    sync::{Condvar, Mutex, MutexGuard},
    time::{Duration, Instant},
};

pub(super) struct BoundedSlotPool<T> {
    slots: Box<[Mutex<Slot<T>>]>,
    coordinator: Mutex<Coordinator>,
    slot_available: Condvar,
}

struct Coordinator {
    available: VecDeque<usize>,
    waiters: VecDeque<u64>,
    next_ticket: u64,
}

struct Slot<T> {
    value: Option<T>,
    valid: bool,
    invalidated_by_unwind: bool,
}

pub(super) struct BoundedSlotLease<'a, T> {
    pool: &'a BoundedSlotPool<T>,
    slot_index: usize,
    slot: Option<MutexGuard<'a, Slot<T>>>,
    wait: Duration,
}

struct WaitTicket<'a, T> {
    pool: &'a BoundedSlotPool<T>,
    ticket: u64,
    active: bool,
}

impl<T> BoundedSlotPool<T> {
    pub(super) fn new(capacity: NonZeroUsize) -> Self {
        let slots = (0..capacity.get())
            .map(|_| {
                Mutex::new(Slot {
                    value: None,
                    valid: false,
                    invalidated_by_unwind: false,
                })
            })
            .collect::<Vec<_>>()
            .into_boxed_slice();
        let available = (0..capacity.get()).collect();

        Self {
            slots,
            coordinator: Mutex::new(Coordinator {
                available,
                waiters: VecDeque::new(),
                next_ticket: 0,
            }),
            slot_available: Condvar::new(),
        }
    }

    pub(super) fn acquire(&self) -> BoundedSlotLease<'_, T> {
        let started = Instant::now();
        let ticket = {
            let mut coordinator = self
                .coordinator
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let ticket = coordinator.next_ticket;
            coordinator.next_ticket = coordinator.next_ticket.wrapping_add(1);
            coordinator.waiters.push_back(ticket);
            ticket
        };
        let mut ticket_guard = WaitTicket {
            pool: self,
            ticket,
            active: true,
        };

        let slot_index = {
            let mut coordinator = self
                .coordinator
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            loop {
                if coordinator.waiters.front() == Some(&ticket)
                    && let Some(slot_index) = coordinator.available.pop_front()
                {
                    let removed = coordinator.waiters.pop_front();
                    debug_assert_eq!(removed, Some(ticket));
                    ticket_guard.active = false;
                    self.slot_available.notify_all();
                    break slot_index;
                }

                coordinator = self
                    .slot_available
                    .wait(coordinator)
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
            }
        };

        let slot = self.slots[slot_index]
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        BoundedSlotLease {
            pool: self,
            slot_index,
            slot: Some(slot),
            wait: started.elapsed(),
        }
    }

    pub(super) fn capacity(&self) -> NonZeroUsize {
        NonZeroUsize::new(self.slots.len()).expect("bounded slot pool capacity is non-zero")
    }

    #[cfg(test)]
    fn waiting_count(&self) -> usize {
        self.coordinator
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .waiters
            .len()
    }
}

impl<T> BoundedSlotLease<'_, T> {
    pub(super) const fn slot_index(&self) -> usize {
        self.slot_index
    }

    pub(super) const fn wait(&self) -> Duration {
        self.wait
    }

    pub(super) fn is_reusable(&self) -> bool {
        let slot = self.slot();
        slot.valid && slot.value.is_some()
    }

    pub(super) fn is_populated(&self) -> bool {
        self.slot().value.is_some()
    }

    pub(super) fn invalidated_by_unwind(&self) -> bool {
        self.slot().invalidated_by_unwind
    }

    pub(super) fn get(&self) -> Option<&T> {
        self.slot().value.as_ref()
    }

    pub(super) fn get_mut(&mut self) -> Option<&mut T> {
        self.slot_mut().value.as_mut()
    }

    pub(super) fn replace(&mut self, value: T) -> Option<T> {
        let slot = self.slot_mut();
        slot.valid = true;
        slot.invalidated_by_unwind = false;
        slot.value.replace(value)
    }

    pub(super) fn take(&mut self) -> Option<T> {
        let slot = self.slot_mut();
        slot.valid = false;
        slot.invalidated_by_unwind = false;
        slot.value.take()
    }

    pub(super) fn invalidate(&mut self) {
        self.slot_mut().valid = false;
    }

    fn slot(&self) -> &Slot<T> {
        self.slot
            .as_deref()
            .expect("bounded slot lease must retain its slot")
    }

    fn slot_mut(&mut self) -> &mut Slot<T> {
        self.slot
            .as_deref_mut()
            .expect("bounded slot lease must retain its slot")
    }
}

impl<T> Drop for BoundedSlotLease<'_, T> {
    fn drop(&mut self) {
        if std::thread::panicking() {
            let slot = self.slot_mut();
            slot.valid = false;
            slot.invalidated_by_unwind = true;
        }
        drop(self.slot.take());

        let mut coordinator = self
            .pool
            .coordinator
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        coordinator.available.push_back(self.slot_index);
        self.pool.slot_available.notify_all();
    }
}

impl<T> Drop for WaitTicket<'_, T> {
    fn drop(&mut self) {
        if !self.active {
            return;
        }
        let mut coordinator = self
            .pool
            .coordinator
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(position) = coordinator
            .waiters
            .iter()
            .position(|ticket| *ticket == self.ticket)
        {
            coordinator.waiters.remove(position);
            self.pool.slot_available.notify_all();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::BoundedSlotPool;
    use std::{
        num::NonZeroUsize,
        panic::{AssertUnwindSafe, catch_unwind},
        sync::{Arc, mpsc},
        thread,
        time::{Duration, Instant},
    };

    const TIMEOUT: Duration = Duration::from_secs(5);

    #[test]
    fn capacity_allows_independent_leases() {
        let pool = BoundedSlotPool::<usize>::new(NonZeroUsize::new(2).unwrap());
        let first = pool.acquire();
        let second = pool.acquire();

        assert_ne!(first.slot_index(), second.slot_index());
    }

    #[test]
    fn exhausted_capacity_waits_until_release() {
        let pool = Arc::new(BoundedSlotPool::<usize>::new(NonZeroUsize::new(1).unwrap()));
        let first = pool.acquire();
        let worker_pool = Arc::clone(&pool);
        let (acquired_tx, acquired_rx) = mpsc::channel();
        let worker = thread::spawn(move || {
            let second = worker_pool.acquire();
            acquired_tx
                .send(second.slot_index())
                .expect("capacity test receiver should remain live");
        });

        wait_for_waiters(&pool, 1);
        assert!(acquired_rx.try_recv().is_err());
        drop(first);

        assert_eq!(
            acquired_rx
                .recv_timeout(TIMEOUT)
                .expect("waiting lease should acquire after release"),
            0,
        );
        worker.join().expect("capacity worker should not panic");
    }

    #[test]
    fn waiter_tickets_are_served_in_fifo_order() {
        let pool = Arc::new(BoundedSlotPool::<usize>::new(NonZeroUsize::new(1).unwrap()));
        let held = pool.acquire();
        let (order_tx, order_rx) = mpsc::channel();

        let first_pool = Arc::clone(&pool);
        let first_tx = order_tx.clone();
        let first = thread::spawn(move || {
            let _lease = first_pool.acquire();
            first_tx.send(1).expect("order receiver should remain live");
            thread::sleep(Duration::from_millis(20));
        });
        wait_for_waiters(&pool, 1);

        let second_pool = Arc::clone(&pool);
        let second = thread::spawn(move || {
            let _lease = second_pool.acquire();
            order_tx.send(2).expect("order receiver should remain live");
        });
        wait_for_waiters(&pool, 2);
        drop(held);

        assert_eq!(order_rx.recv_timeout(TIMEOUT).unwrap(), 1);
        assert_eq!(order_rx.recv_timeout(TIMEOUT).unwrap(), 2);
        first.join().expect("first waiter should not panic");
        second.join().expect("second waiter should not panic");
    }

    #[test]
    fn unwind_invalidates_but_preserves_slot_value_for_safe_teardown() {
        let pool = BoundedSlotPool::new(NonZeroUsize::new(1).unwrap());

        let panic = catch_unwind(AssertUnwindSafe(|| {
            let mut lease = pool.acquire();
            lease.replace(42);
            panic!("synthetic leased-test panic");
        }));
        assert!(panic.is_err());

        let lease = pool.acquire();
        assert!(!lease.is_reusable());
        assert!(lease.is_populated());
        assert!(lease.invalidated_by_unwind());
        assert_eq!(lease.get(), Some(&42));
    }

    fn wait_for_waiters<T>(pool: &BoundedSlotPool<T>, expected: usize) {
        let started = Instant::now();
        while pool.waiting_count() < expected {
            assert!(started.elapsed() < TIMEOUT, "waiter did not enter queue");
            thread::yield_now();
        }
    }
}
