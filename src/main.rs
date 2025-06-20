use std::time::{Duration, Instant};
use guest::prove_state_transition;
use tracing::{error, info};
mod cli;
use cli::operation::OperationName;
use clap::Parser;
use ream_lib::{file::read_file, input::OperationInput,ssz::to_ssz};
use ream_consensus::electra::{beacon_block::SignedBeaconBlock, beacon_state::BeaconState};
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Argument for STFs
    #[clap(flatten)]
    fork: cli::fork::ForkArgs,

    #[clap(flatten)]
    operation: cli::operation::OperationArgs,

    #[clap(long)]
    excluded_cases: Vec<String>,
}
pub fn main() {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }
    // Initialize tracing. In order to view logs, run `RUST_LOG=info cargo run`
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::filter::EnvFilter::from_default_env())
        .init();
    let test_case_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("mainnet")
        .join("tests")
        .join("mainnet");
    if !std::path::Path::new(&test_case_dir).exists() {
        error!("Error: You must first download test data via `make download`");
        std::process::exit(1);
    }
    let args = Args::parse();
    let fork = args.fork.fork;
    let operation_name = args.operation.operation_name;
    let excluded_cases = args.excluded_cases;

    let base_dir = test_case_dir
        .join(format!("{}", fork))
        .join("operations")
        .join(format!("{}", operation_name))
        .join("pyspec_tests");
    let test_cases = ream_lib::file::get_test_cases(&base_dir);
    println!("{:?}",test_cases);
    for test_case in test_cases {
        if excluded_cases.contains(&test_case) {
            info!("Skipping test case: {}", test_case);
            continue;
        }
        info!("{}", "-".repeat(50));
        info!("[{}] Test case: {}", operation_name, test_case);

        let case_dir = &base_dir.join(&test_case);
        let input_path = &case_dir.join(format!("{}.ssz_snappy", operation_name.to_input_name()));
        let pre_state: BeaconState = read_file(&case_dir.join("pre.ssz_snappy"));

        // let pre_state: Vec<u8> = ssz_from_file(&case_dir.join("pre.ssz_snappy"));
        let input = match operation_name {
            OperationName::Attestation => OperationInput::Attestation(read_file(input_path)),
            OperationName::AttesterSlashing => {
                OperationInput::AttesterSlashing(read_file(input_path))
            }
            OperationName::BlockHeader => OperationInput::BeaconBlock(read_file(input_path)),
            OperationName::BLSToExecutionChange => {
                OperationInput::SignedBLSToExecutionChange(read_file(input_path))
            }
            OperationName::Deposit => OperationInput::Deposit(read_file(input_path)),
            OperationName::ExecutionPayload => {
                OperationInput::BeaconBlockBody(read_file(input_path))
            }
            OperationName::ProposerSlashing => {
                OperationInput::ProposerSlashing(read_file(input_path))
            }
            OperationName::SyncAggregate => {
                OperationInput::SyncAggregate(read_file(input_path))
            }
            OperationName::VoluntaryExit => {
                OperationInput::SignedVoluntaryExit(read_file(input_path))
            }
            OperationName::Withdrawals => {
                OperationInput::ExecutionPayload(read_file(input_path))
            }
        };
        let target_dir = "/tmp/jolt-guest-targets";
        let start = Instant::now();
        let program = guest::compile_state_transition(target_dir);
        let end = Instant::now();
        let duration_compile=end.duration_since(start); 
        let start = Instant::now();
        let prover_preprocessing = guest::preprocess_prover_state_transition(&program);
        let end = Instant::now();
        let duration_preprocessing=end.duration_since(start);
        let verifier_preprocessing = guest::preprocess_verifier_state_transition(&program);
        let start=Instant::now();
        let prove_state_transition = guest::build_prover_state_transition(program, prover_preprocessing);
        let end = Instant::now();
        let duration_config=end.duration_since(start);
        let verify_state_transition = guest::build_verifier_state_transition(verifier_preprocessing);
        let start=Instant::now();
        let (output, proof) = prove_state_transition(pre_state.clone(),&input);
        let end=Instant::now();
        let duration_proving=end.duration_since(start);
        
        let is_valid = verify_state_transition(pre_state,&input, output, proof);

        println!("output: {:?}",output);
        println!("valid: {is_valid}");
    }
}
