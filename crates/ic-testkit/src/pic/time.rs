use pocket_ic::{PocketIc, Time};

/// Multi-step time helpers that add test policy beyond PocketIC's primitives.
pub trait PocketIcTimeExt {
    /// Advance PocketIC by a fixed number of execution rounds.
    fn tick_n(&self, times: usize);

    /// Capture PocketIC wall-clock time as nanoseconds since the Unix epoch.
    fn current_time_nanos(&self) -> u64;

    /// Restore wall-clock and certified time from one captured value.
    fn restore_time_nanos(&self, nanos_since_epoch: u64);
}

impl PocketIcTimeExt for PocketIc {
    fn tick_n(&self, times: usize) {
        for _ in 0..times {
            self.tick();
        }
    }

    fn current_time_nanos(&self) -> u64 {
        self.get_time().as_nanos_since_unix_epoch()
    }

    fn restore_time_nanos(&self, nanos_since_epoch: u64) {
        let restored = Time::from_nanos_since_unix_epoch(nanos_since_epoch);
        self.set_time(restored);
        self.set_certified_time(restored);
    }
}
