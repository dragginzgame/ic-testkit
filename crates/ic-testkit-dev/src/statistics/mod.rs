//! Private finite-sample projections for the fixture report.

use core::fmt;

#[cfg(test)]
mod tests;

/// Approximate floating-point projections of a complete finite sample slice.
///
/// The count is exact. Mean and median use floating-point arithmetic; the
/// nearest-rank quantile is an observed value, not a histogram bucket range.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SampleStatistics {
    /// Number of admitted observations, including measured zeroes.
    pub count: usize,
    /// Sum of each observation divided by the sample count.
    pub mean: f64,
    /// Middle observation, or the midpoint of the two middle observations.
    pub median: f64,
    /// Observed value at the caller-selected nearest rank.
    pub quantile: f64,
}

/// Why a complete sample slice cannot supply the requested projection.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SampleStatisticsError {
    /// The quantile must satisfy `0 < numerator <= denominator`.
    InvalidQuantile,
    /// A sample is NaN or infinite; admission fails before sorting.
    NonFiniteSample {
        /// Original zero-based position of the first invalid sample.
        index: usize,
    },
    /// Floating-point summation overflowed; the sample slice remains sorted.
    NonFiniteMean,
}

impl fmt::Display for SampleStatisticsError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidQuantile => {
                formatter.write_str("quantile must be greater than zero and at most one")
            }
            Self::NonFiniteSample { index } => write!(formatter, "sample {index} is not finite"),
            Self::NonFiniteMean => formatter.write_str("sample mean is not finite"),
        }
    }
}

impl core::error::Error for SampleStatisticsError {}

/// Sort finite samples in place and project a mean, median and nearest-rank quantile.
///
/// Consumers own sample collection, units, memory, attribution and the quantile
/// fraction. This function neither samples a platform nor allocates storage.
/// Negative values and signed zeroes are admitted; empty samples return `None`.
/// The quantile uses `ceil(count * numerator / denominator)`, computed with
/// integer arithmetic. Use `95, 100` for p95. Even-count medians use a midpoint
/// that avoids overflow and preserves equal subnormal observations.
///
/// # Errors
/// An invalid fraction or non-finite sample refuses before changing the slice.
/// A non-finite mean returns an error after sorting. Counts remain exact, while
/// floating-point projections and the conversion of the count for division are
/// approximate. No histogram or exact-integer aggregate is reconstructed.
///
#[allow(clippy::cast_precision_loss)]
pub fn summarize_samples(
    samples: &mut [f64],
    numerator: u64,
    denominator: u64,
) -> Result<Option<SampleStatistics>, SampleStatisticsError> {
    if numerator == 0 || numerator > denominator {
        return Err(SampleStatisticsError::InvalidQuantile);
    }
    for (index, sample) in samples.iter().enumerate() {
        if !sample.is_finite() {
            return Err(SampleStatisticsError::NonFiniteSample { index });
        }
    }
    if samples.is_empty() {
        return Ok(None);
    }
    samples.sort_unstable_by(f64::total_cmp);
    let count = samples.len();
    let mean = samples
        .iter()
        .map(|sample| sample / count as f64)
        .sum::<f64>();
    if !mean.is_finite() {
        return Err(SampleStatisticsError::NonFiniteMean);
    }
    let middle = count / 2;
    let median = if count.is_multiple_of(2) {
        samples[middle - 1].midpoint(samples[middle])
    } else {
        samples[middle]
    };
    // usize is at most 64 bits on the supported native/Wasm targets. The wide
    // product retains exact ranks even when numerator is near u64::MAX.
    let rank = ((count as u128) * u128::from(numerator)).div_ceil(u128::from(denominator));
    let index = (rank - 1) as usize;
    Ok(Some(SampleStatistics {
        count,
        mean,
        median,
        quantile: samples[index],
    }))
}
