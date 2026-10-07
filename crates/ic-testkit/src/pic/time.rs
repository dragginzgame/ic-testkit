use pocket_ic::PocketIc;
use std::{fmt, task::Poll, time::Duration};

/// A caller-owned readiness check failed or exhausted its simulated round budget.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TickUntilError<E> {
    /// The predicate was still pending after this many advance/tick rounds.
    ProgressLimit { rounds: u32 },
    /// The predicate returned its own failure.
    Failed(E),
}

impl<E: fmt::Display> fmt::Display for TickUntilError<E> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ProgressLimit { rounds } => {
                write!(
                    formatter,
                    "readiness remained pending after {rounds} rounds"
                )
            }
            Self::Failed(error) => write!(formatter, "readiness check failed: {error}"),
        }
    }
}

impl<E: std::error::Error + 'static> std::error::Error for TickUntilError<E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::ProgressLimit { .. } => None,
            Self::Failed(error) => Some(error),
        }
    }
}

/// Check a caller-owned predicate, advancing simulated time and ticking while pending.
///
/// The predicate runs once before any mutation. Each pending result permits one
/// round of `advance_time(advance)`, then `tick()`, then another predicate check,
/// up to `max_rounds`. A zero budget still performs the initial check. A zero
/// duration permits ticks without advancing time. Completion on the last allowed
/// round succeeds; predicate failures return immediately without another round.
///
/// This bounds simulated progression, not wall-clock time spent inside PocketIC
/// calls or the predicate. PocketIC operation failures retain upstream panic
/// behavior. Applications own readiness meaning and predicate error types.
pub fn tick_until<T, E>(
    pic: &PocketIc,
    max_rounds: u32,
    advance: Duration,
    mut poll: impl FnMut(&PocketIc) -> Poll<Result<T, E>>,
) -> Result<T, TickUntilError<E>> {
    poll_with_rounds(
        max_rounds,
        || {
            pic.advance_time(advance);
            pic.tick();
        },
        || poll(pic),
    )
}

fn poll_with_rounds<T, E>(
    max_rounds: u32,
    mut round: impl FnMut(),
    mut poll: impl FnMut() -> Poll<Result<T, E>>,
) -> Result<T, TickUntilError<E>> {
    let mut rounds = 0;
    loop {
        match poll() {
            Poll::Ready(result) => return result.map_err(TickUntilError::Failed),
            Poll::Pending if rounds == max_rounds => {
                return Err(TickUntilError::ProgressLimit { rounds });
            }
            Poll::Pending => {
                round();
                rounds += 1;
            }
        }
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    #[test]
    fn initial_completion_and_failure_do_not_advance() {
        for budget in [0, 3] {
            for result in [Ok(42), Err("predicate failure")] {
                assert_eq!(
                    poll_with_rounds(
                        budget,
                        || panic!("unexpected round"),
                        || Poll::Ready(result)
                    ),
                    result.map_err(TickUntilError::Failed)
                );
            }
        }
    }

    #[test]
    fn completion_on_last_round_preserves_poll_order() {
        let rounds = Cell::new(0);
        let polls = Cell::new(0);
        let result = poll_with_rounds(
            3,
            || {
                assert_eq!(polls.get(), rounds.get() + 1);
                rounds.set(rounds.get() + 1);
            },
            || {
                assert_eq!(polls.get(), rounds.get());
                polls.set(polls.get() + 1);
                if rounds.get() == 3 {
                    Poll::Ready(Ok::<_, ()>(42))
                } else {
                    Poll::Pending
                }
            },
        );
        assert_eq!(result, Ok(42));
        assert_eq!((rounds.get(), polls.get()), (3, 4));
    }

    #[test]
    fn exhaustion_performs_exactly_the_allowed_rounds() {
        for budget in [0, 1, 3] {
            let rounds = Cell::new(0);
            let polls = Cell::new(0);
            let result = poll_with_rounds(
                budget,
                || rounds.set(rounds.get() + 1),
                || {
                    polls.set(polls.get() + 1);
                    Poll::<Result<(), ()>>::Pending
                },
            );
            assert_eq!(
                result,
                Err(TickUntilError::ProgressLimit { rounds: budget })
            );
            assert_eq!((rounds.get(), polls.get()), (budget, budget + 1));
        }
    }

    #[test]
    fn failure_after_progress_preserves_the_original_cause() {
        let rounds = Cell::new(0);
        let result = poll_with_rounds(
            3,
            || rounds.set(rounds.get() + 1),
            || {
                if rounds.get() == 1 {
                    Poll::Ready(Err::<(), _>(std::io::Error::from(
                        std::io::ErrorKind::PermissionDenied,
                    )))
                } else {
                    Poll::Pending
                }
            },
        );
        let error = result.unwrap_err();
        let source = std::error::Error::source(&error).unwrap();
        assert_eq!(
            source.downcast_ref::<std::io::Error>().unwrap().kind(),
            std::io::ErrorKind::PermissionDenied
        );
        assert_eq!(rounds.get(), 1);
    }
}
