use prefixity_controlled_benchmark::{
    certify_preserved_calibration_evidence, certify_stage1_reasoning_off_preserved_evidence,
    validate_stage1_reasoning_off_evidence,
};
use serde_json::json;

fn main() {
    let result = (|| -> Result<serde_json::Value, String> {
        Ok(json!({
            "state": "PRESERVED_EVIDENCE_VALIDATED",
            "stage1_smoke01": certify_stage1_reasoning_off_preserved_evidence()
                .map_err(|error| error.to_string())?,
            "stage1_smoke02": validate_stage1_reasoning_off_evidence()
                .map_err(|error| error.to_string())?,
            "reasoning_budget_calibration": certify_preserved_calibration_evidence()
                .map_err(|error| error.to_string())?,
            "network_calls": 0,
            "inference_requests": 0
        }))
    })();

    match result {
        Ok(value) => println!(
            "{}",
            serde_json::to_string_pretty(&value).expect("certification result serializes")
        ),
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    }
}
