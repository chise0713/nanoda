#[derive(Debug, thiserror::Error)]
pub enum ChaseSeqError {
    #[error("size is zero")]
    SizeIsZero,
}

#[derive(Debug, thiserror::Error)]
pub enum ChaseSeqBuilderError {
    #[error("size is zero")]
    SizeIsZero,
}
