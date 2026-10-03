use prefixity_controlled_benchmark::phase1c_claim2_tokenization::{
    execute_plan_with_transport, new_pass_evidence, plan_summary, PassEvidence, RawHttpResponse,
    TokenizationTransport,
};
use prefixity_controlled_benchmark::phase1c_claim2_v2_tokenization::{
    preflight, repository_root, OfflinePreflight, IDENTITY, MATERIALIZATION_COMMIT,
};
use reqwest::blocking::{Client, Response};
use reqwest::redirect::Policy;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::Duration;

const MODEL_PATH: &str = r"D:\Prefixity-Lab\models\Qwen3.5-9B\Qwen3.5-9B-Q4_K_M.gguf";
const MODEL_SHA256: &str = "cd76ec205963b3b33350093e6904d9de16c4e666fd104e1f632d25c7f15f2a13";
const LLAMA_PATH: &str = r"C:\Users\USER\AppData\Local\Microsoft\WindowsApps\llama.exe";
const LLAMA_SHA256: &str = "cbe0655558e73168b3bc73f61aa70ec224475152b44c022f6e837616704d0617";
const LLAMA_BUILD: &str = "b10217-ddd4ec142";
const READINESS_URL: &str = "http://127.0.0.1:8080/health";
const INPUT_TOKENS_URL: &str = "http://127.0.0.1:8080/v1/chat/completions/input_tokens";
const MAX_RESPONSE_BYTES: usize = 65_536;

fn usage() -> &'static str {
    "usage:\n  prefixity-phase1c-claim2-v2-tokenization preflight\n  prefixity-phase1c-claim2-v2-tokenization execute --mode HYBRID_8_NEW|FULL_FRESH_22 --operator-started --model-path PATH --model-sha256 SHA256 --llama-path PATH --llama-sha256 SHA256 --llama-build BUILD --context 8192 --slots 1 --reasoning off --offline --host 127.0.0.1 --port 8080 --evidence NEW_PATH"
}

fn parse_execute_confirmation(
    args: &[String],
) -> Result<(BTreeMap<String, String>, PathBuf, String), String> {
    let mut values = BTreeMap::new();
    let mut started = false;
    let mut offline = false;
    let mut index = 1;
    while index < args.len() {
        let flag = &args[index];
        if flag == "--operator-started" {
            if started {
                return Err("duplicate --operator-started".into());
            }
            started = true;
            index += 1;
            continue;
        }
        if flag == "--offline" {
            if offline {
                return Err("duplicate --offline".into());
            }
            offline = true;
            index += 1;
            continue;
        }
        if !flag.starts_with("--") {
            return Err(format!("unexpected argument {flag}"));
        }
        let value = args
            .get(index + 1)
            .ok_or_else(|| format!("missing value for {flag}"))?
            .clone();
        if values.insert(flag.clone(), value).is_some() {
            return Err(format!("duplicate argument {flag}"));
        }
        index += 2;
    }
    if !started || !offline {
        return Err("execute requires --operator-started and --offline confirmations".into());
    }
    if values.keys().any(|key| {
        !matches!(
            key.as_str(),
            "--model-path"
                | "--model-sha256"
                | "--llama-path"
                | "--llama-sha256"
                | "--llama-build"
                | "--context"
                | "--slots"
                | "--reasoning"
                | "--host"
                | "--port"
                | "--evidence"
                | "--mode"
        )
    }) {
        return Err("execute accepts no endpoint, plan, ledger, model, or runtime override".into());
    }
    let expected = [
        ("--model-path", MODEL_PATH),
        ("--model-sha256", MODEL_SHA256),
        ("--llama-path", LLAMA_PATH),
        ("--llama-sha256", LLAMA_SHA256),
        ("--llama-build", LLAMA_BUILD),
        ("--context", "8192"),
        ("--slots", "1"),
        ("--reasoning", "off"),
        ("--host", "127.0.0.1"),
        ("--port", "8080"),
    ];
    for (key, expected_value) in expected {
        if values.get(key).map(String::as_str) != Some(expected_value) {
            return Err(format!(
                "{key} must exactly match the accepted runtime identity"
            ));
        }
    }
    let evidence = values
        .get("--evidence")
        .map(PathBuf::from)
        .ok_or_else(|| "--evidence NEW_PATH is required".to_owned())?;
    if values.len() != expected.len() + 2 {
        return Err("execute requires the complete fixed runtime and mode confirmation".into());
    }
    let mode = values
        .get("--mode")
        .ok_or_else(|| "--mode is required".to_owned())?;
    if !matches!(mode.as_str(), "HYBRID_8_NEW" | "FULL_FRESH_22") {
        return Err("--mode must name one frozen V2 plan".into());
    }
    Ok((values.clone(), evidence, mode.clone()))
}

fn ensure_llama_arg_environment_empty() -> Result<(), String> {
    for (key, _) in env::vars_os() {
        if key.to_string_lossy().starts_with("LLAMA_ARG_") {
            return Err("LLAMA_ARG_* environment variables must be absent before contact".into());
        }
    }
    Ok(())
}

fn sha256_file(path: &Path) -> Result<String, String> {
    let mut file = fs::File::open(path)
        .map_err(|error| format!("cannot open registered runtime artifact {path:?}: {error}"))?;
    let mut hasher = Sha256::new();
    let mut buffer = vec![0_u8; 1024 * 1024];
    loop {
        let read = file.read(&mut buffer).map_err(|error| {
            format!("cannot read registered runtime artifact {path:?}: {error}")
        })?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn verify_runtime_artifact_identities() -> Result<(), String> {
    let model_hash = sha256_file(Path::new(MODEL_PATH))?;
    if model_hash != MODEL_SHA256 {
        return Err("registered GGUF SHA-256 does not match the accepted artifact".into());
    }
    let llama_hash = sha256_file(Path::new(LLAMA_PATH))?;
    if llama_hash != LLAMA_SHA256 {
        return Err("registered llama.exe SHA-256 does not match the accepted build".into());
    }
    Ok(())
}

fn verify_frozen_experiment_identity(
    offline: &OfflinePreflight,
) -> Result<serde_json::Value, String> {
    let root = repository_root();
    let bytes =
        fs::read(root.join(IDENTITY)).map_err(|e| format!("V2 identity unavailable: {e}"))?;
    let identity: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    let seal_path = root.join("docs/phase-1/PHASE_1C_CLAIM_2_TOKENIZATION_IDENTITY_V2.sha256");
    let seal = fs::read_to_string(seal_path).map_err(|e| e.to_string())?;
    let canonical = serde_json::to_vec(&identity).map_err(|e| e.to_string())?;
    let canonical_hash = format!("{:x}", Sha256::digest(canonical));
    let executable = env::current_exe().map_err(|e| e.to_string())?;
    if seal.split_whitespace().next() != Some(canonical_hash.as_str())
        || identity["schema_id"] != "prefixity.phase1c.claim2-tokenization-v2-identity"
        || identity["bindings"]["materialization_commit"] != MATERIALIZATION_COMMIT
        || identity["bindings"]["artifact_hashes"]["v2_ledger_sha256"]
            != offline.hybrid.ledger_sha256
        || identity["bindings"]["plans"]["hybrid_sha256"] != offline.hybrid.plan_sha256
        || identity["bindings"]["plans"]["full_fresh_sha256"] != offline.full_fresh.plan_sha256
        || identity["bindings"]["instrument_identity"]["gguf_sha256"] != MODEL_SHA256
        || identity["bindings"]["instrument_identity"]["llama_sha256"] != LLAMA_SHA256
        || identity["bindings"]["instrument_identity"]["llama_build"] != LLAMA_BUILD
        || identity["bindings"]["zero_inference_rule"]["inference_allowance"] != 0
        || identity["bindings"]["plan_selection_rule"]["before_first_contact"] != true
        || identity["bindings"]["client_executable"]["sha256"] != sha256_file(&executable)?
        || identity["bindings"]["client_executable"]["bytes"]
            != fs::metadata(&executable).map_err(|e| e.to_string())?.len()
    {
        return Err("V2 sealed identity or frozen executable mismatch".into());
    }
    Ok(identity)
}

fn prepare_evidence_path(path: &Path) -> Result<PathBuf, String> {
    if path.exists() {
        return Err("evidence destination must be a new file".into());
    }
    let file_name = path
        .file_name()
        .ok_or_else(|| "evidence path must name a file".to_owned())?;
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let parent = parent
        .canonicalize()
        .map_err(|error| format!("evidence parent directory is unavailable: {error}"))?;
    let target = parent.join(file_name);
    if target.exists() {
        return Err("evidence destination must be a new file".into());
    }
    Ok(target)
}

fn write_evidence(path: &Path, evidence: &PassEvidence) -> Result<(), String> {
    let mut bytes = serde_json::to_vec_pretty(evidence)
        .map_err(|error| format!("could not encode partial tokenization evidence: {error}"))?;
    bytes.push(b'\n');
    fs::write(path, bytes).map_err(|error| format!("could not persist partial evidence: {error}"))
}

fn bounded_response(response: Response) -> Result<RawHttpResponse, String> {
    let status = response.status().as_u16();
    let mut reader = response.take((MAX_RESPONSE_BYTES + 1) as u64);
    let mut body = Vec::new();
    reader
        .read_to_end(&mut body)
        .map_err(|error| format!("could not read bounded endpoint response: {error}"))?;
    let truncated = body.len() > MAX_RESPONSE_BYTES;
    if truncated {
        body.truncate(MAX_RESPONSE_BYTES);
    }
    Ok(RawHttpResponse {
        status,
        body,
        truncated,
    })
}

struct LocalTokenizationTransport {
    client: Client,
}

impl LocalTokenizationTransport {
    fn new() -> Result<Self, String> {
        let client = Client::builder()
            .no_proxy()
            .redirect(Policy::none())
            .connect_timeout(Duration::from_secs(2))
            .timeout(Duration::from_secs(15))
            .build()
            .map_err(|error| format!("could not create fixed local HTTP client: {error}"))?;
        Ok(Self { client })
    }
}

impl TokenizationTransport for LocalTokenizationTransport {
    fn readiness(&mut self) -> Result<RawHttpResponse, String> {
        let response = self
            .client
            .get(READINESS_URL)
            .header(reqwest::header::ACCEPT, "application/json")
            .send()
            .map_err(|error| format!("single readiness contact failed: {error}"))?;
        bounded_response(response)
    }

    fn input_tokens(&mut self, exact_body: &[u8]) -> Result<RawHttpResponse, String> {
        let response = self
            .client
            .post(INPUT_TOKENS_URL)
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .header(reqwest::header::ACCEPT, "application/json")
            .body(exact_body.to_vec())
            .send()
            .map_err(|error| format!("single input-token contact failed: {error}"))?;
        bounded_response(response)
    }
}

fn run_preflight() -> Result<(), String> {
    let offline = preflight(&repository_root()).map_err(|error| error.to_string())?;
    let identity = verify_frozen_experiment_identity(&offline)?;
    verify_runtime_artifact_identities()?;
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "schema_id": "prefixity.phase1c.claim2-tokenization-v2-offline-preflight",
            "hybrid": plan_summary(&offline.hybrid),
            "full_fresh": plan_summary(&offline.full_fresh),
            "inheritance_eligible": offline.inheritance_eligible,
            "inheritance_reason": offline.inheritance_reason,
            "experiment_id": identity["experiment_id"],
            "server_contacts": 0,
            "inference_requests": 0
        }))
        .expect("summary serializes")
    );
    Ok(())
}

fn run_execute(args: &[String]) -> Result<(), String> {
    let (confirmation, evidence_path, mode) = parse_execute_confirmation(args)?;
    ensure_llama_arg_environment_empty()?;
    let evidence_path = prepare_evidence_path(&evidence_path)?;

    // All pinned source files and exact bodies are validated before the HTTP
    // client is built. The accepted model and llama.exe artifacts are also
    // re-hashed before contact. The client has no process-startup API.
    let offline = preflight(&repository_root()).map_err(|error| error.to_string())?;
    verify_runtime_artifact_identities()?;
    let identity = verify_frozen_experiment_identity(&offline)?;
    let selected_mode = if offline.inheritance_eligible {
        "HYBRID_8_NEW"
    } else {
        "FULL_FRESH_22"
    };
    if mode != selected_mode {
        return Err(format!(
            "pre-contact plan selection is {selected_mode}; requested {mode}"
        ));
    }
    let inputs = if mode == "HYBRID_8_NEW" {
        offline.hybrid
    } else {
        offline.full_fresh
    };
    let inherited_count_mappings = if mode == "HYBRID_8_NEW" {
        identity["bindings"]["inherited_count_mappings"].clone()
    } else {
        serde_json::Value::Null
    };
    let mut evidence = new_pass_evidence(&inputs);
    evidence.schema_id = "prefixity.phase1c.claim2-tokenization-v2-pass-evidence".into();
    evidence.schema_version = 2;
    evidence.operator_runtime_confirmation = json!({
        "selected_mode": mode,
        "selection_basis": "pre-contact sealed inheritance and accepted runtime identity",
        "experiment_identity": identity["experiment_id"],
        "inherited_count_mappings": inherited_count_mappings,
        "operator_started_server": true,
        "model_path": confirmation.get("--model-path"),
        "model_sha256": confirmation.get("--model-sha256"),
        "llama_executable_path": confirmation.get("--llama-path"),
        "llama_sha256": confirmation.get("--llama-sha256"),
        "llama_build": confirmation.get("--llama-build"),
        "reasoning": confirmation.get("--reasoning"),
        "context_tokens": confirmation.get("--context"),
        "slots": confirmation.get("--slots"),
        "offline": true,
        "host": confirmation.get("--host"),
        "port": confirmation.get("--port")
    });
    write_evidence(&evidence_path, &evidence)?;

    let mut transport = LocalTokenizationTransport::new()?;
    let result = execute_plan_with_transport(&inputs, &mut transport, |updated| {
        // Carry the operator's explicit identity confirmation into each
        // durable partial-evidence rewrite.
        let mut updated = updated.clone();
        updated.schema_id = "prefixity.phase1c.claim2-tokenization-v2-pass-evidence".into();
        updated.schema_version = 2;
        updated.operator_runtime_confirmation = evidence.operator_runtime_confirmation.clone();
        write_evidence(&evidence_path, &updated)
    })
    .map_err(|error| error.to_string())?;
    evidence = result;
    evidence.schema_id = "prefixity.phase1c.claim2-tokenization-v2-pass-evidence".into();
    evidence.schema_version = 2;
    evidence.operator_runtime_confirmation = json!({
        "selected_mode": mode,
        "selection_basis": "pre-contact sealed inheritance and accepted runtime identity",
        "experiment_identity": identity["experiment_id"],
        "inherited_count_mappings": inherited_count_mappings,
        "operator_started_server": true,
        "model_path": confirmation.get("--model-path"),
        "model_sha256": confirmation.get("--model-sha256"),
        "llama_executable_path": confirmation.get("--llama-path"),
        "llama_sha256": confirmation.get("--llama-sha256"),
        "llama_build": confirmation.get("--llama-build"),
        "reasoning": confirmation.get("--reasoning"),
        "context_tokens": confirmation.get("--context"),
        "slots": confirmation.get("--slots"),
        "offline": true,
        "host": confirmation.get("--host"),
        "port": confirmation.get("--port")
    });
    write_evidence(&evidence_path, &evidence)?;
    println!(
        "{}",
        serde_json::to_string_pretty(&evidence).expect("evidence serializes")
    );
    if evidence.terminal_classification.is_some() {
        return Err("TOKENIZATION_PASS_INCONCLUSIVE; partial evidence was retained".into());
    }
    Ok(())
}

fn main() {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let result = match args.first().map(String::as_str) {
        Some("preflight") if args.len() == 1 => run_preflight(),
        Some("execute") => run_execute(&args),
        _ => Err(usage().into()),
    };
    if let Err(error) = result {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::{
        parse_execute_confirmation, LLAMA_BUILD, LLAMA_PATH, LLAMA_SHA256, MODEL_PATH, MODEL_SHA256,
    };

    fn accepted_arguments() -> Vec<String> {
        [
            "execute",
            "--mode",
            "HYBRID_8_NEW",
            "--operator-started",
            "--model-path",
            MODEL_PATH,
            "--model-sha256",
            MODEL_SHA256,
            "--llama-path",
            LLAMA_PATH,
            "--llama-sha256",
            LLAMA_SHA256,
            "--llama-build",
            LLAMA_BUILD,
            "--context",
            "8192",
            "--slots",
            "1",
            "--reasoning",
            "off",
            "--offline",
            "--host",
            "127.0.0.1",
            "--port",
            "8080",
            "--evidence",
            "evidence.json",
        ]
        .into_iter()
        .map(str::to_owned)
        .collect()
    }

    #[test]
    fn execute_cli_accepts_only_the_fixed_runtime_and_no_endpoint_selector() {
        let accepted = accepted_arguments();
        assert!(parse_execute_confirmation(&accepted).is_ok());

        let mut generation_path = accepted_arguments();
        generation_path.splice(
            1..1,
            ["--endpoint", "/v1/chat/completions"].map(str::to_owned),
        );
        assert!(parse_execute_confirmation(&generation_path).is_err());

        let mut arbitrary_host = accepted_arguments();
        let host_index = arbitrary_host
            .iter()
            .position(|arg| arg == "127.0.0.1")
            .unwrap();
        arbitrary_host[host_index] = "example.invalid".into();
        assert!(parse_execute_confirmation(&arbitrary_host).is_err());

        let mut no_operator_confirmation = accepted_arguments();
        no_operator_confirmation.retain(|arg| arg != "--operator-started");
        assert!(parse_execute_confirmation(&no_operator_confirmation).is_err());

        let mut invalid_mode = accepted_arguments();
        invalid_mode[2] = "TRY_THEN_EXPAND".into();
        assert!(parse_execute_confirmation(&invalid_mode).is_err());
    }
}
