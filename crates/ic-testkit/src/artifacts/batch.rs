pub(super) fn indexed_outcomes<T, E>(
    results: &[Result<T, E>],
) -> impl Iterator<Item = (usize, &T)> {
    results
        .iter()
        .enumerate()
        .filter_map(|(index, result)| result.as_ref().ok().map(|outcome| (index, outcome)))
}

pub(super) fn indexed_failures<T, E>(
    results: &[Result<T, E>],
) -> impl Iterator<Item = (usize, &E)> {
    results
        .iter()
        .enumerate()
        .filter_map(|(index, result)| result.as_ref().err().map(|error| (index, error)))
}
