use crate::{ChaseSeqBuilder, scale_iterations};

#[test]
fn test_scale_iterations_miri() {
    assert_eq!(scale_iterations(123), 5);
}

#[test]
fn test_miri() {
    let builder = ChaseSeqBuilder::default().size(4).unwrap();
    let chase_seq = builder.build();
    assert_eq!(chase_seq.size, 4 * crate::KB / crate::PTR_SIZE);
    assert_eq!(chase_seq.num_iter, 5);
    chase_seq.chase(5).unwrap();
}
