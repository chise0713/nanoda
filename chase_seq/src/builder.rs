use crate::{ChaseSeq, errors::ChaseSeqBuilderError};

/// Builder for `ChaseSeq`.
#[derive(Default)]
pub struct ChaseSeqBuilder(ChaseSeq);

impl ChaseSeqBuilder {
    /// Set the size in KiB of memory region to chase.
    pub fn size(mut self, size: usize) -> Result<Self, ChaseSeqBuilderError> {
        if size == 0 {
            return Err(ChaseSeqBuilderError::SizeIsZero);
        }
        _ = self.0.set_size(size);
        Ok(self)
    }

    /// Set whether to use CPU fence on each pointer dereference.
    pub fn fence(mut self, fence: bool) -> Self {
        self.0.set_fence(fence);
        self
    }

    /// Build the `ChaseSeq`.
    pub fn build(self) -> Result<ChaseSeq, ChaseSeqBuilderError> {
        if self.0.size() == 0 {
            return Err(ChaseSeqBuilderError::SizeIsZero);
        }
        Ok(self.0)
    }
}
