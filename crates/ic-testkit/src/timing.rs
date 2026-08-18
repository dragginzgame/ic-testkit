use std::time::Duration;

pub const fn saturating_add_optional_duration(
    left: Option<Duration>,
    right: Option<Duration>,
) -> Option<Duration> {
    match (left, right) {
        (None, None) => None,
        (Some(duration), None) | (None, Some(duration)) => Some(duration),
        (Some(left), Some(right)) => Some(left.saturating_add(right)),
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::saturating_add_optional_duration;

    #[test]
    fn optional_duration_addition_preserves_absence_and_saturates() {
        assert_eq!(saturating_add_optional_duration(None, None), None);
        assert_eq!(
            saturating_add_optional_duration(Some(Duration::from_secs(2)), None),
            Some(Duration::from_secs(2))
        );
        assert_eq!(
            saturating_add_optional_duration(Some(Duration::MAX), Some(Duration::from_nanos(1))),
            Some(Duration::MAX)
        );
    }
}
