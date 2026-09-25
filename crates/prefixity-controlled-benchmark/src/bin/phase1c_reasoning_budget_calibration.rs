use prefixity_controlled_benchmark::{
    attempt_002_exclusivity_preflight, attempt_003_poststart, attempt_005_poststart,
    attempt_006_poststart, attempt_007_poststart, certify_workflow_identity, dry_run_attempt_002,
    dry_run_attempt_003, dry_run_attempt_004, dry_run_attempt_005, dry_run_attempt_006,
    dry_run_attempt_007, dry_run_attempt_008, dry_run_attempt_009, dry_run_calibration,
    execute_attempt_002, execute_attempt_003, execute_attempt_007, execute_attempt_008,
    execute_attempt_009, execute_calibration, fingerprint_attempt_002, fingerprint_attempt_003,
    fingerprint_attempt_004, fingerprint_attempt_005, fingerprint_attempt_006,
    fingerprint_attempt_007, fingerprint_attempt_008, fingerprint_attempt_009,
    fingerprint_calibration, freeze_attempt_008, freeze_attempt_009, parse_calibration_cli_args,
    preflight_attempt_002, preflight_attempt_003, preflight_attempt_004, preflight_attempt_005,
    preflight_attempt_006, preflight_attempt_007, preflight_attempt_008, preflight_attempt_009,
    preflight_calibration, summarize_attempt_002_budget, summarize_calibration_budget,
    validate_attempt_007_execution_evidence, validate_attempt_008_preparation,
    validate_attempt_009_candidate_order, validate_attempt_009_preparation, CalibrationCliCommand,
};
use prefixity_controlled_benchmark::{
    dry_run_attempt_010, execute_attempt_010, fingerprint_attempt_010, freeze_attempt_010,
    preflight_attempt_010, validate_attempt_010_live_prerequisites,
    validate_attempt_010_preparation, validate_attempt_010_repository_contract,
};
use std::env;

fn main() {
    let result = match parse_calibration_cli_args(env::args().skip(1)) {
        Ok(CalibrationCliCommand::Preflight) => preflight_calibration(),
        Ok(CalibrationCliCommand::Fingerprint) => fingerprint_calibration(),
        Ok(CalibrationCliCommand::DryRun) => dry_run_calibration(),
        Ok(CalibrationCliCommand::Run { budget }) => execute_calibration(budget, true),
        Ok(CalibrationCliCommand::Summarize { budget }) => summarize_calibration_budget(budget),
        Ok(CalibrationCliCommand::Attempt002Fingerprint) => fingerprint_attempt_002(),
        Ok(CalibrationCliCommand::Attempt002Preflight) => preflight_attempt_002(),
        Ok(CalibrationCliCommand::Attempt002DryRun) => dry_run_attempt_002(),
        Ok(CalibrationCliCommand::Attempt002ExclusivityPreflight) => {
            attempt_002_exclusivity_preflight(true)
        }
        Ok(CalibrationCliCommand::RunAttempt002 { budget, server_pid }) => {
            execute_attempt_002(budget, server_pid, true, true)
        }
        Ok(CalibrationCliCommand::SummarizeAttempt002 { budget }) => {
            summarize_attempt_002_budget(budget)
        }
        Ok(CalibrationCliCommand::Attempt003Fingerprint) => fingerprint_attempt_003(),
        Ok(CalibrationCliCommand::Attempt003Preflight) => preflight_attempt_003(),
        Ok(CalibrationCliCommand::Attempt003DryRun) => dry_run_attempt_003(),
        Ok(CalibrationCliCommand::Attempt003Poststart) => attempt_003_poststart(),
        Ok(CalibrationCliCommand::RunAttempt003) => execute_attempt_003(),
        Ok(CalibrationCliCommand::Attempt004Fingerprint) => fingerprint_attempt_004(),
        Ok(CalibrationCliCommand::Attempt004Preflight) => preflight_attempt_004(),
        Ok(CalibrationCliCommand::Attempt004DryRun) => dry_run_attempt_004(),
        Ok(CalibrationCliCommand::Attempt005Fingerprint) => fingerprint_attempt_005(),
        Ok(CalibrationCliCommand::Attempt005Preflight) => preflight_attempt_005(),
        Ok(CalibrationCliCommand::Attempt005DryRun) => dry_run_attempt_005(),
        Ok(CalibrationCliCommand::Attempt005Poststart) => attempt_005_poststart(),
        Ok(CalibrationCliCommand::Attempt006Fingerprint) => fingerprint_attempt_006(),
        Ok(CalibrationCliCommand::Attempt006Preflight) => preflight_attempt_006(),
        Ok(CalibrationCliCommand::Attempt006DryRun) => dry_run_attempt_006(),
        Ok(CalibrationCliCommand::Attempt006Poststart) => attempt_006_poststart(),
        Ok(CalibrationCliCommand::Attempt007Fingerprint) => fingerprint_attempt_007(),
        Ok(CalibrationCliCommand::Attempt007Preflight) => preflight_attempt_007(),
        Ok(CalibrationCliCommand::Attempt007DryRun) => dry_run_attempt_007(),
        Ok(CalibrationCliCommand::Attempt007Poststart) => attempt_007_poststart(),
        Ok(CalibrationCliCommand::Attempt007ValidateEvidence) => {
            validate_attempt_007_execution_evidence()
        }
        Ok(CalibrationCliCommand::RunAttempt007) => execute_attempt_007(),
        Ok(CalibrationCliCommand::Attempt008Fingerprint) => fingerprint_attempt_008(),
        Ok(CalibrationCliCommand::Attempt008Preflight) => preflight_attempt_008(),
        Ok(CalibrationCliCommand::Attempt008DryRun) => dry_run_attempt_008(),
        Ok(CalibrationCliCommand::Attempt008Freeze) => freeze_attempt_008(),
        Ok(CalibrationCliCommand::Attempt008ValidatePreparation) => {
            validate_attempt_008_preparation()
        }
        Ok(CalibrationCliCommand::RunAttempt008) => execute_attempt_008(),
        Ok(CalibrationCliCommand::Attempt009Fingerprint) => fingerprint_attempt_009(),
        Ok(CalibrationCliCommand::Attempt009Preflight) => preflight_attempt_009(),
        Ok(CalibrationCliCommand::Attempt009DryRun) => dry_run_attempt_009(),
        Ok(CalibrationCliCommand::Attempt009Freeze) => freeze_attempt_009(),
        Ok(CalibrationCliCommand::Attempt009ValidatePreparation) => {
            validate_attempt_009_preparation()
        }
        Ok(CalibrationCliCommand::Attempt009CandidateOrder) => {
            validate_attempt_009_candidate_order()
        }
        Ok(CalibrationCliCommand::RunAttempt009) => execute_attempt_009(),
        Ok(CalibrationCliCommand::Attempt010Fingerprint) => fingerprint_attempt_010(),
        Ok(CalibrationCliCommand::Attempt010Preflight) => preflight_attempt_010(),
        Ok(CalibrationCliCommand::Attempt010DryRun) => dry_run_attempt_010(),
        Ok(CalibrationCliCommand::Attempt010Freeze) => freeze_attempt_010(),
        Ok(CalibrationCliCommand::Attempt010ValidatePreparation) => {
            validate_attempt_010_preparation()
        }
        Ok(CalibrationCliCommand::Attempt010RepositoryContract) => {
            validate_attempt_010_repository_contract()
        }
        Ok(CalibrationCliCommand::Attempt010LivePrerequisites) => {
            validate_attempt_010_live_prerequisites()
        }
        Ok(CalibrationCliCommand::RunAttempt010) => execute_attempt_010(),
        Ok(CalibrationCliCommand::WorkflowIdentityCertification { result_path }) => {
            certify_workflow_identity(&result_path)
        }
        Err(error) => Err(error),
    };
    match result {
        Ok(value) => println!(
            "{}",
            serde_json::to_string_pretty(&value).expect("calibration result serializes")
        ),
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    }
}
