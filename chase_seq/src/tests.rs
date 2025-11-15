use crate::{ChaseSeqBuilder, ChaseSeqBuilderError, ChaseSeqError, CommonError};

#[test]
fn test_zero_size() {
    let result = ChaseSeqBuilder::default().size(0);
    assert!(matches!(
        result,
        Err(ChaseSeqBuilderError::CommonError(CommonError::SizeIsZero))
    ));
}

#[test]
fn test_large_size() {
    let large_size = usize::MAX;
    let result = ChaseSeqBuilder::default().size(large_size);
    assert!(matches!(
        result,
        Err(ChaseSeqBuilderError::CommonError(CommonError::SizeTooLarge))
    ));
}

#[test]
fn test_large_test_iterations() {
    let builder = ChaseSeqBuilder::default().size(64).unwrap();
    let chase_seq = builder.build();
    let large_iterations = usize::MAX;
    let result = chase_seq.chase(large_iterations);
    assert!(matches!(result, Err(ChaseSeqError::TestIterationsTooLarge)));
}

#[test]
fn test_valid_size() {
    let builder = ChaseSeqBuilder::default().size(64).unwrap();
    let chase_seq = builder.build();
    assert_eq!(chase_seq.size(), 64);
}

#[test]
fn test_fence_setting() {
    let builder = ChaseSeqBuilder::default().fence(true);
    let chase_seq = builder.build();
    assert!(chase_seq.fence());
}

#[test]
fn test_seed_setting() {
    let seed_str = "test_seed";
    let builder = ChaseSeqBuilder::default().seed(seed_str);
    let chase_seq = builder.build();
    assert_eq!(chase_seq.seed, seed_str);
}

#[test]
fn test_copy_clone() {
    let builder = ChaseSeqBuilder::default()
        .size(128)
        .unwrap()
        .fence(true)
        .seed("seed");
    let chase_seq1 = builder.build();
    let chase_seq2 = chase_seq1;
    let chase_seq3 = chase_seq1.clone();
    assert_eq!(chase_seq1.size(), chase_seq2.size());
    assert_eq!(chase_seq1.fence(), chase_seq3.fence());
    assert_eq!(chase_seq1.seed, chase_seq2.seed);
}
