use crate::{configure::*, types::*};

/// # Top-P sampling
/// This sampler adds up the token probabilities until the value is
/// greater or equal to `p` and at least `min_keep` tokens have been
/// encountered. The remaining tokens are eliminated.
///
/// **Properties**:
/// - Filters logits
///
/// **Parameters**:
/// - `min_keep`: Minimum number of entries to keep. (default: `1`)
/// - `p`: Target value. (default: `0.9`)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SampleTopP {
    pub(crate) p: L,
    pub(crate) min_keep: usize,
}

impl Default for SampleTopP {
    fn default() -> Self {
        Self {
            p: 0.9f32,
            min_keep: 1,
        }
    }
}

impl SampleTopP {
    pub fn new(p: L, min_keep: usize) -> Self {
        Self { p, min_keep }
    }

    pub fn min_keep(mut self, val: usize) -> Self {
        self.min_keep = val;
        self
    }

    pub fn p(mut self, val: L) -> Self {
        self.p = val;
        self
    }
}

impl Sampler for SampleTopP {
    fn sample<'a>(
        &mut self,
        _res: &mut dyn HasSamplerResources,
        logits: &'a mut Logits,
    ) -> anyhow::Result<&'a mut Logits, SamplerError> {
        use std::ops::ControlFlow::*;

        let Self { p, min_keep } = *self;
        logits.ensure_softmax().map_err(|e| {
            SamplerError::InternalError(format!("Failed to ensure softmax before sampling: {}", e))
        })?;

        let mut cum_sum = 0f32;
        let last_idx =
            match logits
                .iter()
                .enumerate()
                .try_fold(logits.len(), |last_idx, (idx, logit)| {
                    cum_sum += logit.prob;
                    if cum_sum >= p && idx + 1 >= min_keep {
                        return Break(idx + 1);
                    }
                    Continue(last_idx)
                }) {
                Continue(i) => i,
                Break(i) => i,
            };
        if last_idx != logits.len() {
            logits.truncate(last_idx);
            logits.set_softmax(false);
        }
        Ok(logits)
    }
}

impl ConfigurableSampler<usize, L> for SampleTopP {}

impl HasSamplerMetadata<usize, L> for SampleTopP {
    fn sampler_metadata(&self) -> SamplerMetadata {
        SamplerMetadata {
            name: "top-p",
            description: Some(concat!(
                "This sampler adds up the token probabilities until the value is ",
                "greater or equal to p and at least min_keep tokens have been encountered.",
                " The remaining tokens are eliminated."
            )),
            options: vec![
                SamplerOptionMetadata {
                    key: "p",
                    description: Some("Target value for cumulative probabilities."),
                    option_type: SamplerOptionType::Float,
                },
                SamplerOptionMetadata {
                    key: "min_keep",
                    description: Some(concat!(
                        "Minimum number of tokens to keep after sampling. ",
                        "Setting this to 0 is not recommended."
                    )),
                    option_type: SamplerOptionType::UInt,
                },
            ],
        }
    }

    fn sampler_options_mut(&mut self) -> SamplerOptions<SamplerOptionValueMut<'_, usize, L>> {
        unsafe {
            SamplerOptions::build_options(
                self.sampler_metadata().options,
                [
                    Some(SamplerOptionValueMut::Float(&mut self.p)),
                    Some(SamplerOptionValueMut::UInt(&mut self.min_keep)),
                ],
            )
        }
    }

    fn sampler_options(&self) -> SamplerOptions<SamplerOptionValue<'_, usize, L>> {
        unsafe {
            SamplerOptions::build_options(
                self.sampler_metadata().options,
                [
                    Some(SamplerOptionValue::Float(self.p)),
                    Some(SamplerOptionValue::UInt(self.min_keep)),
                ],
            )
        }
    }
}
