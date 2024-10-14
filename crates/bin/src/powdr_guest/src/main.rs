use powdr_riscv_runtime::io::read;

use sbv::primitives::zk_trie::db::NodeDb;
use sbv::{
    core::{EvmExecutorBuilder, HardforkConfig, VerificationError},
    cycle_track, dev_error, dev_info, dev_trace, measure_duration_millis,
    primitives::{types::BlockTrace, zk_trie::db::kv::HashMapDb, Block},
    update_metrics_counter,
};

fn main() {
    verify().unwrap();
}

fn verify() -> Result<(), VerificationError> {
    let l2_trace: BlockTrace = read(1);
    let fork_config: HardforkConfig = read(2);
    dev_trace!("{l2_trace:#?}");
    let root_after = l2_trace.root_after();

    let mut zktrie_db = cycle_track!(
        {
            let mut zktrie_db = NodeDb::new(HashMapDb::default());
            measure_duration_millis!(
                build_zktrie_db_duration_milliseconds,
                l2_trace.build_zktrie_db(&mut zktrie_db).unwrap()
            );
            zktrie_db
        },
        "build ZktrieState"
    );

    let mut executor = EvmExecutorBuilder::new(HashMapDb::default(), &mut zktrie_db)
        .hardfork_config(fork_config)
        .build(&l2_trace)?;

    // TODO: change to Result::inspect_err when sp1 toolchain >= 1.76
    #[allow(clippy::map_identity)]
    #[allow(clippy::manual_inspect)]
    executor.handle_block(&l2_trace).map_err(|e| {
        dev_error!(
            "Error occurs when executing block #{}({:?}): {e:?}",
            l2_trace.number(),
            l2_trace.block_hash()
        );

        update_metrics_counter!(verification_error);
        e
    })?;
    let revm_root_after = executor.commit_changes()?;

    if root_after != revm_root_after {
        dev_error!(
                "Block #{}({:?}) root mismatch: root after in trace = {root_after:x}, root after in revm = {revm_root_after:x}",
                l2_trace.number(),
                l2_trace.block_hash(),
            );

        update_metrics_counter!(verification_error);

        return Err(VerificationError::RootMismatch {
            root_trace: root_after,
            root_revm: revm_root_after,
        });
    }
    dev_info!(
        "Block #{}({}) verified successfully",
        l2_trace.number(),
        l2_trace.block_hash(),
    );

    Ok(())
}
