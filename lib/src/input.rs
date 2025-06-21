use ream_consensus::{
    attestation::Attestation,
    attester_slashing::AttesterSlashing,
    bls_to_execution_change::SignedBLSToExecutionChange,
    deposit::Deposit,
    electra::{
        beacon_block::BeaconBlock, beacon_block_body::BeaconBlockBody,
        execution_payload::ExecutionPayload,
    },
    proposer_slashing::ProposerSlashing,
    sync_aggregate::SyncAggregate,
    voluntary_exit::SignedVoluntaryExit,
};
use serde::{Deserialize, Serialize};
use ssz_derive::{Decode, Encode};

#[derive(Serialize, Deserialize, Clone, Debug, Encode, Decode)]
pub enum OperationInput {
    Attestation(Attestation),
    AttesterSlashing(AttesterSlashing),
    BeaconBlock(BeaconBlock),
    SignedBLSToExecutionChange(SignedBLSToExecutionChange),
    Deposit(Deposit),
    BeaconBlockBody(BeaconBlockBody),
    ProposerSlashing(ProposerSlashing),
    SyncAggregate(SyncAggregate),
    SignedVoluntaryExit(SignedVoluntaryExit),
    ExecutionPayload(ExecutionPayload),
}
