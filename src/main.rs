use clap::Parser;
use guest::prove_state_transition;
use std::path::PathBuf;
use tracing::{error, info};

mod cli;
use cli::{fork::Fork, operation::OperationName};

use std::time::{Duration, Instant};

use ream_consensus::electra::beacon_state::BeaconState;
use ream_lib::{file::ssz_from_file, input::OperationInput, ssz::from_ssz_bytes};

/// The arguments for the command.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Argument for STFs
    #[clap(flatten)]
    fork: cli::fork::ForkArgs,

    #[clap(flatten)]
    operation: cli::operation::OperationArgs,

    /// Verify the correctness of the state root by comparing against consensus-spec-tests' post_state
    #[clap(long, default_value_t = false)]
    compare_specs: bool,

    /// Verify the correctness of the state root by recomputing on the host
    #[clap(long, default_value_t = true)]
    compare_recompute: bool,

    #[clap(long)]
    excluded_cases: Vec<String>,
}
pub fn main() {
    setup_log();

    let (fork, operation_name, excluded_cases, compare_specs, compare_recompute) = parse_args();
    let (base_dir, test_cases) = load_test_cases(&fork, &operation_name);

    for test_case in test_cases {
        if excluded_cases.contains(&test_case) {
            info!("Skipping test case: {test_case}");
            continue;
        }

        info!("[{operation_name}] Test case: {test_case}");

        let case_dir = &base_dir.join(&test_case);
        let input = prepare_input(&case_dir, &operation_name);
        let pre_state_ssz_bytes: Vec<u8> = ssz_from_file(&case_dir.join("pre.ssz_snappy"));
        println!("pre_state size: {} bytes", pre_state_ssz_bytes.len());

        let target_dir = "/tmp/jolt-guest-targets";

        let start = Instant::now();
        let program = guest::compile_state_transition(target_dir);
        let end = Instant::now();
        let duration_compile = end.duration_since(start);
        println!("duration compile : {:?}", duration_compile);
        let start = Instant::now();

        let output = guest::state_transition(pre_state_ssz_bytes.clone(), input.clone());
        let end = Instant::now();
        let duration_compile = end.duration_since(start);
        println!("duration compile : {:?}", duration_compile);


        // println!("{:?}", output); // ProgramSummary does not implement Debug

        // let start = Instant::now();
        // let prover_preprocessing = guest::preprocess_prover_state_transition(&program);
        // let end = Instant::now();
        // let duration_prover_preprocessing = end.duration_since(start);
        // println!("duration compile : {:?}", duration_prover_preprocessing);

        // let start = Instant::now();
        // let verifier_preprocessing = guest::preprocess_verifier_state_transition(&program);
        // let end = Instant::now();
        // let duration_verifier_preprocessing = end.duration_since(start);
        // println!("duration compile : {:?}", duration_verifier_preprocessing);

        // let start = Instant::now();
        // let prove_state_transition =
        //     guest::build_prover_state_transition(program, prover_preprocessing);
        // let end = Instant::now();
        // let duration_config = end.duration_since(start);

        // let verify_state_transition =
        //     guest::build_verifier_state_transition(verifier_preprocessing);
        // let start = Instant::now();
        // let (output, proof) = prove_state_transition(pre_state_ssz_bytes.clone(), input.clone());
        // let end = Instant::now();
        // let duration_proving = end.duration_since(start);

        // let is_valid =
        //     verify_state_transition(pre_state_ssz_bytes.clone(), input, output.clone(), proof);
        assert_state_matches_specs(&output, &pre_state_ssz_bytes, case_dir);
        // println!("output: {:?}", output);
        // println!("valid: {is_valid}");
    }
}
fn setup_log() {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }

    // Initialize tracing. In order to view logs, run `RUST_LOG=info cargo run`
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::filter::EnvFilter::from_default_env())
        .init();
}
fn parse_args() -> (Fork, OperationName, Vec<String>, bool, bool) {
    let args = Args::parse();
    (
        args.fork.fork,
        args.operation.operation_name,
        args.excluded_cases,
        args.compare_specs,
        args.compare_recompute,
    )
}
fn prepare_input(case_dir: &PathBuf, operation_name: &OperationName) -> OperationInput {
    let input_path = &case_dir.join(format!("{}.ssz_snappy", operation_name.to_input_name()));
    let ssz_bytes = ssz_from_file(input_path);

    match operation_name {
        OperationName::Attestation => OperationInput::Attestation(ssz_bytes),
        OperationName::AttesterSlashing => OperationInput::AttesterSlashing(ssz_bytes),
        OperationName::BlockHeader => OperationInput::BeaconBlock(ssz_bytes),
        OperationName::BLSToExecutionChange => {
            OperationInput::SignedBLSToExecutionChange(ssz_bytes)
        }
        OperationName::Deposit => OperationInput::Deposit(ssz_bytes),
        OperationName::ExecutionPayload => OperationInput::BeaconBlockBody(ssz_bytes),
        OperationName::ProposerSlashing => OperationInput::ProposerSlashing(ssz_bytes),
        OperationName::SyncAggregate => OperationInput::SyncAggregate(ssz_bytes),
        OperationName::VoluntaryExit => OperationInput::SignedVoluntaryExit(ssz_bytes),
        OperationName::Withdrawals => OperationInput::ExecutionPayload(ssz_bytes),
    }
}

fn load_test_cases(fork: &Fork, operation_name: &OperationName) -> (PathBuf, Vec<String>) {
    // These assets are from consensus-specs repo.
    let test_case_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("mainnet")
        .join("tests")
        .join("mainnet");

    if !std::path::Path::new(&test_case_dir).exists() {
        error!("Error: You must first download test data via `make download`");
        std::process::exit(1);
    }

    let base_dir = test_case_dir
        .join(format!("{}", fork))
        .join("operations")
        .join(format!("{}", operation_name))
        .join("pyspec_tests");

    let test_cases = ream_lib::file::get_test_cases(&base_dir);

    (base_dir, test_cases)
}
fn assert_state_matches_specs(
    new_state: &BeaconState,
    pre_state_ssz_bytes: &[u8],
    case_dir: &PathBuf,
) {
    let post_state_opt: Option<BeaconState> = {
        if case_dir.join("post.ssz_snappy").exists() {
            let ssz_bytes: Vec<u8> = ssz_from_file(&case_dir.join("post.ssz_snappy"));
            Some(from_ssz_bytes(&ssz_bytes).unwrap())
        } else {
            None
        }
    };
    match post_state_opt {
        // If the specs provide post_state, compare the computed root against post_state's root
        Some(post_state) => {
            info!("post_state provided. The state root should be mutated.");
            assert_eq!(*new_state, post_state);
            info!("Execution is correct! State mutated and the roots match.");
        }
        // If the specs does not contain a post_state, compare the computed root against pre_state's root
        None => {
            info!("post_state not provided. The state root should not be mutated.");
            let pre_state: BeaconState = from_ssz_bytes(&pre_state_ssz_bytes).unwrap();
            assert_eq!(*new_state, pre_state);
            info!("Execution is correct! State should not be mutated and the roots match.");
        }
    }
}
