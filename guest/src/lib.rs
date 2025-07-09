// #![cfg_attr(feature = "guest", no_std)]
// use tree_hash::TreeHash;

// use std::os::macos::raw::stat;

use ream_consensus::{
    attestation::Attestation,
    attester_slashing::AttesterSlashing,
    bls_to_execution_change::SignedBLSToExecutionChange,
    deposit::Deposit,
    electra::{
        beacon_block::BeaconBlock, beacon_state::BeaconState, execution_payload::ExecutionPayload,
    },
    proposer_slashing::ProposerSlashing,
    sync_aggregate::SyncAggregate,
    voluntary_exit::SignedVoluntaryExit,
};
use ream_lib::{input::OperationInput, ssz::deserialize};

use getrandom::register_custom_getrandom;
fn custom_getrandom(buf: &mut [u8]) -> Result<(), getrandom::Error> {
    // Fill with zeros or any deterministic bytes for benchmarking.
    for b in buf.iter_mut() {
        *b = 0;
    }
    Ok(())
}
register_custom_getrandom!(custom_getrandom);

#[jolt::provable(
    stack_size = 100000,
    memory_size = 50000000,
    max_input_size = 10000000,
    max_output_size = 10000000
)]
fn state_transition(pre_state_ssz_bytes: Vec<u8>, input: OperationInput) -> BeaconState {
    let mut state: BeaconState = deserialize(&pre_state_ssz_bytes);

    match input {
        OperationInput::Attestation(ssz_bytes) => {
            let attestation: Attestation = deserialize(&ssz_bytes);
            let _ = state.process_attestation(&attestation);
        }
        OperationInput::AttesterSlashing(ssz_bytes) => {
            let attester_slashing: AttesterSlashing = deserialize(&ssz_bytes);
            let _ = state.process_attester_slashing(&attester_slashing);
        }
        OperationInput::BeaconBlock(ssz_bytes) => {
            let block: BeaconBlock = deserialize(&ssz_bytes);
            let _ = state.process_block_header(&block);
        }
        OperationInput::SignedBLSToExecutionChange(ssz_bytes) => {
            let bls_change: SignedBLSToExecutionChange = deserialize(&ssz_bytes);
            let _ = state.process_bls_to_execution_change(&bls_change);
        }
        OperationInput::Deposit(ssz_bytes) => {
            let deposit: Deposit = deserialize(&ssz_bytes);
            let _ = state.process_deposit(&deposit);
        }
        OperationInput::BeaconBlockBody(_ssz_bytes) => {
            panic!("Not implemented");
            // let block_body: BeaconBlockBody = deserialize(&ssz_bytes);
            // let _ = state.process_execution_payload(&block_body);
        }
        OperationInput::ProposerSlashing(ssz_bytes) => {
            let proposer_slashing: ProposerSlashing = deserialize(&ssz_bytes);
            let _ = state.process_proposer_slashing(&proposer_slashing);
        }
        OperationInput::SyncAggregate(ssz_bytes) => {
            let sync_aggregate: SyncAggregate = deserialize(&ssz_bytes);
            let _ = state.process_sync_aggregate(&sync_aggregate);
        }
        OperationInput::SignedVoluntaryExit(ssz_bytes) => {
            let voluntary_exit: SignedVoluntaryExit = deserialize(&ssz_bytes);
            let _ = state.process_voluntary_exit(&voluntary_exit);
        }
        OperationInput::ExecutionPayload(ssz_bytes) => {
            let execution_payload: ExecutionPayload = deserialize(&ssz_bytes);
            let _ = state.process_withdrawals(&execution_payload);
        }
    }
    state
}
