use pocket_ic::PocketIc;

/// Focused time conversion missing from PocketIC's native API.
///
/// All mutation, certified-time, and round operations stay on [`PocketIc`].
pub trait PocketIcTimeExt {
    /// Read PocketIC wall-clock time as nanoseconds since the Unix epoch.
    fn current_time_nanos(&self) -> u64;
}

impl PocketIcTimeExt for PocketIc {
    fn current_time_nanos(&self) -> u64 {
        self.get_time().as_nanos_since_unix_epoch()
    }
}
