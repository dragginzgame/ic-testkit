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

/// Advance simulated time once, then deliver exactly the caller-selected ticks.
///
/// Zero duration performs no time read or write; zero ticks permits advancement
/// without delivering a round. This function does not poll, retry, start a server
/// or decide application readiness. Callers own completion and cadence budgets.
///
/// # Panics
/// PocketIC operation failures retain the upstream client's panic behavior.
pub fn advance_and_tick(pic: &PocketIc, advance: Duration, ticks: u32) {
    if !advance.is_zero() {
        pic.advance_time(advance);
    }
    for _ in 0..ticks {
        pic.tick();
    }
}

/// Check a caller-owned predicate, advancing simulated time and ticking while pending.
///
/// The predicate runs once before any mutation. Each pending result permits one
/// round of `advance_time(advance)`, then `tick()`, then another predicate check,
/// up to `max_rounds`. A zero budget still performs the initial check. A zero
/// duration permits ticks without reading or writing simulated time. Completion
/// on the last allowed round succeeds; predicate failures return immediately without another round.
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
        || advance_and_tick(pic, advance, 1),
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
    #[test]
    fn polling_transport_skips_zero_time_and_preserves_nonzero_advancement() {
        for advance in [Duration::ZERO, Duration::from_nanos(7)] {
            assert_time_requests(advance, 1, |pic| {
                let mut polls = 0;
                let result = tick_until(pic, 1, advance, |_| {
                    polls += 1;
                    if polls == 2 {
                        Poll::Ready(Ok::<_, ()>(42))
                    } else {
                        Poll::Pending
                    }
                });
                assert_eq!(result, Ok(42));
                assert_eq!(polls, 2);
            });
        }
    }

    #[test]
    fn tick_batches_preserve_exact_transport_order_and_selected_counts() {
        for advance in [Duration::ZERO, Duration::from_nanos(7)] {
            for ticks in [0, 2, 4, 24] {
                assert_time_requests(advance, ticks, |pic| advance_and_tick(pic, advance, ticks));
            }
        }
    }

    #[test]
    fn install_cooldown_uses_exactly_two_ticks() {
        use crate::pic::CanisterInstallExt as _;
        for advance in [Duration::ZERO, Duration::from_nanos(7)] {
            assert_time_requests(advance, 2, |pic| {
                pic.wait_out_install_code_rate_limit(advance);
            });
        }
    }

    fn assert_time_requests(advance: Duration, ticks: u32, run: impl FnOnce(&PocketIc)) {
        use std::{io::Write as _, net::TcpListener, thread, time::Instant};
        let mut expected = Vec::new();
        if !advance.is_zero() {
            expected.extend(["/instances/0/read/get_time", "/instances/0/update/set_time"]);
        }
        expected.extend(std::iter::repeat_n(
            "/instances/0/update/tick",
            ticks as usize,
        ));
        let request_count = expected.len();
        let listener = TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0)).unwrap();
        listener.set_nonblocking(true).unwrap();
        let url = format!("http://{}/", listener.local_addr().unwrap())
            .parse()
            .unwrap();
        let worker = thread::spawn(move || {
            let deadline = Instant::now() + Duration::from_secs(5);
            let mut requests = Vec::new();
            while requests.len() < request_count {
                assert!(Instant::now() < deadline, "simulated-time peer timed out");
                let (mut stream, _) = match listener.accept() {
                    Ok(connection) => connection,
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(1));
                        continue;
                    }
                    Err(error) => panic!("accept simulated-time request: {error}"),
                };
                let request = ic_host_process::testing::http::capture_http_request(
                    &mut stream,
                    Duration::from_secs(1),
                    16_384,
                    16_384,
                    ic_host_process::testing::http::HttpContentLength::Optional,
                )
                .into_result()
                .unwrap();
                let text = std::str::from_utf8(&request).unwrap();
                let path = text.split_whitespace().nth(1).unwrap().to_owned();
                let reply = if path.ends_with("/read/get_time") {
                    "{\"nanos_since_epoch\":100}"
                } else {
                    "null"
                };
                if path.ends_with("/update/set_time") {
                    let body = text.split_once("\r\n\r\n").unwrap().1;
                    assert_eq!(
                        serde_json::from_str::<serde_json::Value>(body).unwrap()["nanos_since_epoch"],
                        100 + u64::from(advance.subsec_nanos())
                    );
                }
                stream
                    .set_write_timeout(Some(Duration::from_secs(1)))
                    .unwrap();
                write!(stream, "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{reply}", reply.len()).unwrap();
                requests.push(path);
            }
            requests
        });
        let pic = PocketIc::new_from_existing_instance(url, 0, Some(1_000));
        run(&pic);
        assert_eq!(worker.join().unwrap(), expected);
    }
}
