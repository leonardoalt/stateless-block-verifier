use sbv::{
    core::{HardforkConfig, VerificationError},
    primitives::{types::BlockTrace, Block},
};

use powdr::GoldilocksField;
use powdr::Pipeline;

pub fn verify<T: Block + Clone>(
    l2_trace: BlockTrace,
    fork_config: &HardforkConfig,
) -> Result<(), VerificationError> {
    println!("Running powdr...");
    let mut p = Pipeline::<GoldilocksField>::default()
        .from_asm_file("crates/bin/src/powdr_guest.asm".into())
        .add_data(1, &l2_trace)
        .add_data(2, fork_config);

    p.compute_witness().unwrap();

    Ok(())
}
