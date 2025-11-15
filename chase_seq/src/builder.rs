use crate::{ChaseSeq, ChaseSeqError, errors::ChaseSeqBuilderError};

/// Builder for `ChaseSeq`.
#[derive(Default, Clone, Copy)]
pub struct ChaseSeqBuilder(ChaseSeq);

impl ChaseSeqBuilder {
    /// Set the size in KiB of memory region to chase.
    pub fn size(mut self, size: usize) -> Result<Self, ChaseSeqBuilderError> {
        self.0.set_size(size).map_err(|e| match e {
            ChaseSeqError::CommonError(err) => ChaseSeqBuilderError::CommonError(err),
            _ => unreachable!(),
        })?;
        Ok(self)
    }

    /// Set whether to use CPU fence on each pointer dereference.
    pub fn fence(mut self, fence: bool) -> Self {
        self.0.set_fence(fence);
        self
    }

    /// Set the seed for random number generator.
    pub fn seed(mut self, seed: &'static str) -> Self {
        self.0.seed = seed;
        self
    }

    /// Build the `ChaseSeq`.
    pub fn build(self) -> ChaseSeq {
        self.0
    }
}
