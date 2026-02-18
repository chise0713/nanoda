#[derive(Debug, thiserror::Error)]
pub enum CommonError {
    #[error("size is zero")]
    SizeIsZero,
    #[error("size is too large")]
    SizeTooLarge,
}

#[derive(Debug, thiserror::Error)]
pub enum ChaseSeqError {
    #[error("{}", .0)]
    CommonError(
        #[from]
        #[source]
        CommonError,
    ),
    #[error("test_iterations is too large")]
    TestIterationsTooLarge,
}

#[derive(Debug, thiserror::Error)]
pub enum ChaseSeqBuilderError {
    #[error("{}", .0)]
    CommonError(
        #[from]
        #[source]
        CommonError,
    ),
}
