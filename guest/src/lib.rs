// #![cfg_attr(feature = "guest", no_std)]
 
use ream_consensus::{
    electra::beacon_state::BeaconState
};
use ream_lib::input::OperationInput;
use getrandom::register_custom_getrandom;

fn custom_getrandom(buf: &mut [u8]) -> Result<(), getrandom::Error> {
    // Fill with zeros or any deterministic bytes for benchmarking.
    for b in buf.iter_mut() { *b = 0; }
    Ok(())
}
register_custom_getrandom!(custom_getrandom);

#[jolt::provable]
fn state_transition(pre_state:BeaconState,input:&OperationInput){
    // let pre_state:BeaconState=deserialize(&prestate).expect("");
    match input {
        //input enum destructured 
        OperationInput::Attestation(attestation) => {
            // let attestation: Attestation = deserialize(&ssz_bytes);
            let _ = pre_state.clone().process_attestation(&attestation);
        }
        OperationInput::AttesterSlashing(attester_slashing) => {
            // let attester_slashing: AttesterSlashing = deserialize(&ssz_bytes);
            let _ = pre_state.clone().process_attester_slashing(&attester_slashing);
        }
        OperationInput::BeaconBlock(block) => {
            // let block: BeaconBlock = deserialize(&ssz_bytes);
            let _ = pre_state.clone().process_block_header(&block);
        }
        OperationInput::SignedBLSToExecutionChange(bls_change) => {
            // let bls_change: SignedBLSToExecutionChange = deserialize(&ssz_bytes);
            let _ = pre_state.clone().process_bls_to_execution_change(&bls_change);
        }
        OperationInput::Deposit(deposit) => {
            // let deposit: Deposit = deserialize(&ssz_bytes);
            let _ = pre_state.clone().process_deposit(&deposit);
        }
        OperationInput::BeaconBlockBody(_ssz_bytes) => {
            panic!("Not implemented");
            // let block_body: BeaconBlockBody = deserialize(&ssz_bytes);
            // let _ = pre_state.clone().process_execution_payload(&block_body);
        }
        OperationInput::ProposerSlashing(proposer_slashing) => {
            // let proposer_slashing: ProposerSlashing = deserialize(&ssz_bytes);
            let _ = pre_state.clone().process_proposer_slashing(&proposer_slashing);
        }
        OperationInput::SyncAggregate(sync_aggregate) => {
            // let sync_aggregate: SyncAggregate = deserialize(&ssz_bytes);
            let _ = pre_state.clone().process_sync_aggregate(&sync_aggregate);
        }
        OperationInput::SignedVoluntaryExit(voluntary_exit) => {
            // let voluntary_exit: SignedVoluntaryExit = deserialize(&ssz_bytes);
            let _ = pre_state.clone().process_voluntary_exit(&voluntary_exit);
        }
        OperationInput::ExecutionPayload(execution_payload) => {
            // let execution_payload: ExecutionPayload = deserialize(&ssz_bytes);
            let _ = pre_state.clone().process_withdrawals(&execution_payload);
        }
    }
}
