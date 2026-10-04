//! Frozen, one-arm-at-a-time Claim-2 reduced-cohort scored pilot.
//! `preflight` and `next` never create a client or contact a server.
use prefixity_controlled_benchmark::{
    load_claim2_case, Claim2ArmState, Claim2ProjectionMode, Claim2SlotStatus,
};
use reqwest::blocking::Client;
use reqwest::redirect::Policy;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const CASES: [&str; 4] = ["CP02", "CP03", "CP05", "CP06"];
const ARMS: [(&str, Claim2ProjectionMode); 3] = [
    ("BASELINE", Claim2ProjectionMode::Baseline),
    ("NO_OP", Claim2ProjectionMode::NoOp),
    ("INTERVENTION", Claim2ProjectionMode::Intervention),
];
const V1_IDENTITY: &str = "4b7265e8a509829b3109947302ec82c2efec4f05cedd3aeac926324fa2a11f16";
const V1_RAW: &str = "caf62ccaba1130a0e75d55316a1828cdcb17a8df4cc73f839f96f31a6110c264";
const V1_RESULT: &str = "03263b532a13619f9e6e38a447bf984651a712944d7c9aa83df9a86764821040";
const ENDPOINT: &str = "http://127.0.0.1:8080/v1/chat/completions";
const MODEL: &str = r"D:\Prefixity-Lab\models\Qwen3.5-9B\Qwen3.5-9B-Q4_K_M.gguf";
const MODEL_SHA: &str = "cd76ec205963b3b33350093e6904d9de16c4e666fd104e1f632d25c7f15f2a13";
const LLAMA: &str = r"C:\Users\USER\AppData\Local\Microsoft\WindowsApps\llama.exe";
const LLAMA_SHA: &str = "cbe0655558e73168b3bc73f61aa70ec224475152b44c022f6e837616704d0617";

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}
fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn read_json(path: &Path) -> Result<Value, String> {
    serde_json::from_slice(&fs::read(path).map_err(|e| e.to_string())?).map_err(|e| e.to_string())
}
fn file_hash(path: &Path) -> Result<String, String> {
    let mut file = fs::File::open(path).map_err(|e| e.to_string())?;
    let mut hash = Sha256::new();
    let mut buf = vec![0u8; 1024 * 1024];
    loop {
        let n = file.read(&mut buf).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        hash.update(&buf[..n]);
    }
    Ok(format!("{:x}", hash.finalize()))
}
fn source_hash(path: &Path) -> Result<String, String> {
    let source =
        String::from_utf8(fs::read(path).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
    Ok(digest(source.replace("\r\n", "\n").as_bytes()))
}
fn write_new(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|e| format!("{}: {e}", path.display()))?;
    file.write_all(bytes)
        .and_then(|_| file.sync_all())
        .map_err(|e| e.to_string())
}
fn now_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_millis()
}
fn ordinal(index: usize) -> (usize, usize) {
    (index / 3, index % 3)
}
fn evidence_rows() -> Result<(Vec<Value>, Value), String> {
    let root = root();
    let result =
        read_json(&root.join("docs/phase-1/PHASE_1C_CLAIM_2_TOKENIZATION_RESULT_V1.json"))?;
    if result["provenance"]["tokenization_identity_canonical_sha256"] != V1_IDENTITY
        || result["provenance"]["raw_evidence_sha256"] != V1_RAW
        || digest(&serde_json::to_vec(&result).map_err(|e| e.to_string())?) != V1_RESULT
        || fs::read_to_string(
            root.join("docs/phase-1/PHASE_1C_CLAIM_2_TOKENIZATION_RESULT_V1.sha256"),
        )
        .map_err(|e| e.to_string())?
        .split_whitespace()
        .next()
            != Some(V1_RESULT)
    {
        return Err("V1 token-evidence binding differs".into());
    }
    let raw = root.join("claim2-tokenization-pass-evidence-v1.json");
    if file_hash(&raw)? != V1_RAW {
        return Err("V1 raw evidence changed".into());
    }
    let rows = result["authoritative_logical_counts"]
        .as_array()
        .ok_or("V1 counts absent")?
        .iter()
        .filter(|row| CASES.iter().any(|case| row["case_id"] == *case))
        .cloned()
        .collect::<Vec<_>>();
    if rows.len() != 36 {
        return Err("V1 reduced mappings != 36".into());
    }
    Ok((rows, result))
}
fn lookup<'a>(rows: &'a [Value], id: &str) -> Result<&'a Value, String> {
    let mut matches = rows.iter().filter(|row| row["logical_request_id"] == id);
    let one = matches.next().ok_or_else(|| format!("missing {id}"))?;
    if matches.next().is_some() {
        return Err(format!("duplicate {id}"));
    }
    Ok(one)
}
fn check_body(body: &[u8], row: &Value, ledger: &Value, id: &str) -> Result<(), String> {
    let hash = digest(body);
    if row["request_body_sha256"] != hash {
        return Err(format!("V1 body hash mismatch: {id}"));
    }
    let entries = ledger["requests"]
        .as_array()
        .ok_or("ledger requests absent")?;
    let mut matches = entries
        .iter()
        .filter(|entry| entry["logical_request_id"] == id);
    let entry = matches
        .next()
        .ok_or_else(|| format!("ledger missing {id}"))?;
    if matches.next().is_some()
        || entry["request_body_sha256"] != hash
        || entry["future_token_counter_body"]
            .as_str()
            .map(str::as_bytes)
            != Some(body)
    {
        return Err(format!("exact ledger body mismatch: {id}"));
    }
    Ok(())
}
fn plan() -> Result<Value, String> {
    let (rows, _) = evidence_rows()?;
    let root = root();
    let ledger = read_json(&root.join("fixtures/claim2/tokenization-request-ledger-v1.json"))?;
    let domain = read_json(&root.join("fixtures/claim2/advancing-output-domain-v1.json"))?;
    let mut arms = Vec::new();
    let mut hashes = BTreeSet::new();
    let mut totals = BTreeMap::<String, u64>::new();
    for case_id in CASES {
        let case = load_claim2_case(&root.join(format!(
            "fixtures/claim2/{}/case.json",
            case_id.to_lowercase()
        )))
        .map_err(|e| e.to_string())?;
        for (arm_name, mode) in ARMS {
            let mut state = Claim2ArmState::new(&case, mode);
            let mut requests = Vec::new();
            for slot in 1..=3 {
                let rendered = state.render_next(&case).map_err(|e| e.to_string())?;
                let id = format!("{case_id}/{arm_name}/slot-{slot}");
                let row = lookup(&rows, &id)?;
                check_body(&rendered.request_json, row, &ledger, &id)?;
                let hash = digest(&rendered.request_json);
                hashes.insert(hash.clone());
                let tokens = row["input_tokens"].as_u64().ok_or("token count missing")?;
                *totals.entry(arm_name.into()).or_default() += tokens;
                let expected_action = if slot < 3 {
                    domain["transitions"]
                        .as_array()
                        .ok_or("domain absent")?
                        .iter()
                        .find(|point| point["case_id"] == case_id && point["request_slot"] == slot)
                        .ok_or("domain transition absent")?["expected_action_id"]
                        .clone()
                } else {
                    Value::Null
                };
                requests.push(json!({"logical_request_id":id,"slot":slot,"body_sha256":hash,"input_tokens_v1_inherited":tokens,"expected_action_id":expected_action}));
                if slot < 3 {
                    let point = domain["transitions"]
                        .as_array()
                        .ok_or("domain absent")?
                        .iter()
                        .find(|point| point["case_id"] == case_id && point["request_slot"] == slot)
                        .ok_or("domain transition absent")?;
                    let raw = point["canonical_raw_response"]
                        .as_str()
                        .ok_or("canonical action absent")?;
                    if state
                        .record_output(&case, raw.into())
                        .map_err(|e| e.to_string())?
                        .is_some()
                    {
                        return Err("canonical action failed offline replay".into());
                    }
                }
            }
            arms.push(json!({"ordinal":arms.len()+1,"case_id":case_id,"role":if case_id == "CP02" || case_id == "CP03" {"positive"} else {"zero_mutation_control"},"arm":arm_name,"requests":requests,"fresh_operator_server":true,"evidence_directory":format!("arm-{:02}",arms.len()+1),"fixture_sha256":file_hash(&root.join(format!("fixtures/claim2/{}/case.json",case_id.to_lowercase())))?,"evaluator_sha256":case.manifest().evaluation_key_sha256,"intervention_decision":if case_id == "CP02" || case_id == "CP03" {"EXACT_DUPLICATE_PRUNE_ON_SLOT_3"} else {"ZERO_MUTATION"}}));
        }
    }
    if arms.len() != 12
        || hashes.len() != 14
        || totals.get("BASELINE") != Some(&22165)
        || totals.get("NO_OP") != Some(&22165)
        || totals.get("INTERVENTION") != Some(&20282)
    {
        return Err("reduced-cohort count invariant failed".into());
    }
    Ok(
        json!({"schema_id":"prefixity.phase1c.claim2-reduced-scored-pilot-manifest","schema_version":1,
        "study":"CLAIM_2_REDUCED_COHORT_DOWNSTREAM_EFFICACY_FEASIBILITY_PILOT",
        "decision_review_commit":"1f6812983054aa460060d0bbd6e6cfd60ce3666f",
        "v1_identity_seal":V1_IDENTITY,"v1_raw_sha256":V1_RAW,"v1_result_seal":V1_RESULT,
        "arms":arms,"logical_request_ceiling":36,"unique_body_hashes":hashes,"arm_token_totals":totals,
        "advancing_domain_sha256":file_hash(&root.join("fixtures/claim2/advancing-output-domain-v1.json"))?,
        "v1_request_ledger_sha256":file_hash(&root.join("fixtures/claim2/tokenization-request-ledger-v1.json"))?,
        "workload_renderer_source_sha256":source_hash(&root.join("crates/prefixity-controlled-benchmark/src/phase1c_claim2_workload.rs"))?,
        "client_source_sha256":source_hash(&root.join("crates/prefixity-controlled-benchmark/src/bin/phase1c_claim2_reduced_pilot.rs"))?,
        "model_sha256":MODEL_SHA,"llama_sha256":LLAMA_SHA,"llama_build":"b10217-ddd4ec142",
        "runtime":{"context":8192,"slots":1,"reasoning":"off","offline":true,"host":"127.0.0.1","port":8080},
        "request":{"max_tokens":1024,"temperature":0,"top_p":1,"seed":1,"stream":false,"chat_template_kwargs":"ABSENT"},
        "endpoint":ENDPOINT,"retry_count":0,"fresh_server_per_arm":true,
        "scoring_specification":"request slots 1 and 2 require exact canonical expected action; slot 3 uses pinned hidden deterministic evaluator; all three must pass for ARM_TASK_SUCCESS; malformed advancing response ends arm without another dispatch",
        "classification_rules":"positive: BASELINE failure => BASELINE_INCAPABLE; BASELINE success and NO_OP failure => NO_OP_PIPELINE_REGRESSION; both success and INTERVENTION success/failure => DOWNSTREAM_SUCCESS_PRESERVED/REGRESSED; controls compare all three outcomes for CONTROL_OUTCOME_DIVERGENCE",
        "study_rules":"both positives preserved and controls invariant => FEASIBILITY_PASSED; positive treatment regression => REGRESSION_OBSERVED; baseline incapability => INCONCLUSIVE_BASELINE; pipeline/control/evidence inconsistency => INCONCLUSIVE_INTEGRITY",
        "resume_rule":"next wholly unstarted arm only after completed evidence verification; never resend ambiguous request or restart a partial arm"}),
    )
}
fn manifest_bytes() -> Result<Vec<u8>, String> {
    let mut bytes = serde_json::to_vec_pretty(&plan()?).map_err(|e| e.to_string())?;
    bytes.push(b'\n');
    Ok(bytes)
}
fn preflight() -> Result<Value, String> {
    let bytes = manifest_bytes()?;
    let path = root().join("docs/phase-1/PHASE_1C_CLAIM_2_REDUCED_PILOT_MANIFEST.json");
    if fs::read(path).map_err(|e| e.to_string())? != bytes {
        return Err("published manifest does not reproduce".into());
    }
    Ok(
        json!({"status":"OFFLINE_PREFLIGHT_PASS","manifest_sha256":digest(&bytes),"arms":12,"logical_requests_maximum":36,"unique_admitted_bodies":14,"token_mappings":36,"input_tokens_total":64612,"inference_requests":0,"server_contacts":0}),
    )
}
fn next(evidence: &Path) -> Result<Value, String> {
    preflight()?;
    for n in 1..=12 {
        let dir = evidence.join(format!("arm-{n:02}"));
        if !dir.exists() {
            return Ok(json!({"next_ordinal":n,"state":"WHOLLY_UNSTARTED"}));
        }
        if !dir.join("DONE.json").exists() {
            return Err(format!("arm {n:02} incomplete or ambiguous; no replay"));
        }
        let done = read_json(&dir.join("DONE.json"))?;
        if done["ordinal"] != n || done["manifest_sha256"] != preflight()?["manifest_sha256"] {
            return Err(format!("arm {n:02} evidence identity mismatch"));
        }
        let (ci, ai) = ordinal(n - 1);
        if done["case_id"] != CASES[ci] || done["arm"] != ARMS[ai].0 {
            return Err(format!("arm {n:02} order mismatch"));
        }
        let slots = done["slots"].as_array().ok_or("completed slots missing")?;
        let performed = slots
            .iter()
            .filter(|slot| slot["raw_assistant_output"].is_string())
            .count();
        for slot in 1..=performed {
            let stem = format!("slot-{slot}");
            let id = format!("{}/{}/slot-{slot}", CASES[ci], ARMS[ai].0);
            let request =
                fs::read(dir.join(format!("{stem}-request.json"))).map_err(|e| e.to_string())?;
            let planned = read_json(&dir.join(format!("{stem}-PLANNED.json")))?;
            let dispatching = read_json(&dir.join(format!("{stem}-DISPATCHING.json")))?;
            let observed = read_json(&dir.join(format!("{stem}-OBSERVED.json")))?;
            let scored = read_json(&dir.join(format!("{stem}-SCORED.json")))?;
            let raw = read_json(&dir.join(format!("{stem}-raw-http-body.bin")))?;
            let content = raw["choices"][0]["message"]["content"]
                .as_str()
                .ok_or("recorded assistant content missing")?;
            if file_hash(&dir.join(format!("{stem}-raw-http-body.bin")))?
                != observed["raw_response_sha256"]
                || observed["http_status"] != 200
                || planned["logical_request_id"] != id
                || dispatching["logical_request_id"] != id
                || planned["request_sha256"] != digest(&request)
                || dispatching["request_sha256"] != digest(&request)
                || scored["assistant_content"] != content
                || scored["assistant_content_sha256"] != digest(content.as_bytes())
                || slots[slot - 1]["raw_assistant_output"] != content
                || scored["logical_request_id"] != id
            {
                return Err(format!("arm {n:02} raw evidence invalid"));
            }
        }
    }
    if evidence.join("arm-13").exists() {
        return Err("unregistered thirteenth arm evidence exists".into());
    }
    Ok(json!({"state":"ALL_12_ARMS_COMPLETE"}))
}

fn classify_arm_triplet(case_id: &str, done: &[Value]) -> &'static str {
    let outcomes = done
        .iter()
        .map(|value| value["arm_task_success"] == true)
        .collect::<Vec<_>>();
    if case_id == "CP05" || case_id == "CP06" {
        if done[0]["slots"] != done[1]["slots"] || done[0]["slots"] != done[2]["slots"] {
            return "CONTROL_OUTCOME_DIVERGENCE";
        }
        if !outcomes[0] {
            return "BASELINE_INCAPABLE";
        }
        return "CONTROL_OUTCOME_INVARIANT";
    }
    if !outcomes[0] {
        "BASELINE_INCAPABLE"
    } else if !outcomes[1] {
        "NO_OP_PIPELINE_REGRESSION"
    } else if outcomes[2] {
        "DOWNSTREAM_SUCCESS_PRESERVED"
    } else {
        "DOWNSTREAM_SUCCESS_REGRESSED"
    }
}

fn report(evidence: &Path) -> Result<Value, String> {
    if next(evidence)?["state"] != "ALL_12_ARMS_COMPLETE" {
        return Err("all 12 arms must be complete before study interpretation".into());
    }
    let mut classifications = BTreeMap::new();
    for (ci, case_id) in CASES.iter().enumerate() {
        let mut done = Vec::new();
        for ai in 0..3 {
            done.push(read_json(
                &evidence.join(format!("arm-{:02}/DONE.json", ci * 3 + ai + 1)),
            )?);
        }
        classifications.insert(*case_id, classify_arm_triplet(case_id, &done));
    }
    let values = classifications.values().copied().collect::<Vec<_>>();
    let terminal = if values.contains(&"CONTROL_OUTCOME_DIVERGENCE")
        || values.contains(&"NO_OP_PIPELINE_REGRESSION")
    {
        "REDUCED_COHORT_DOWNSTREAM_EFFICACY_INCONCLUSIVE_INTEGRITY"
    } else if values.contains(&"DOWNSTREAM_SUCCESS_REGRESSED") {
        "REDUCED_COHORT_DOWNSTREAM_EFFICACY_REGRESSION_OBSERVED"
    } else if values.contains(&"BASELINE_INCAPABLE") {
        "REDUCED_COHORT_DOWNSTREAM_EFFICACY_INCONCLUSIVE_BASELINE"
    } else if classifications["CP02"] == "DOWNSTREAM_SUCCESS_PRESERVED"
        && classifications["CP03"] == "DOWNSTREAM_SUCCESS_PRESERVED"
    {
        "REDUCED_COHORT_DOWNSTREAM_EFFICACY_FEASIBILITY_PASSED"
    } else {
        "REDUCED_COHORT_DOWNSTREAM_EFFICACY_INCONCLUSIVE_INTEGRITY"
    };
    Ok(json!({"case_classifications":classifications,"terminal_classification":terminal}))
}
fn verify_server(pid: u32) -> Result<String, String> {
    // No shell command or endpoint is supplied by the caller.
    let ps = format!("$p=Get-CimInstance Win32_Process -Filter 'ProcessId={pid}'; $l=@(Get-NetTCPConnection -LocalAddress 127.0.0.1 -LocalPort 8080 -State Listen -ErrorAction Stop); if ($null -eq $p -or $l.Count -ne 1 -or $l[0].OwningProcess -ne {pid} -or $p.ExecutablePath -ne '{LLAMA}' -or $p.CommandLine -ne '\"{LLAMA}\" serve -m {MODEL} -c 8192 -np 1 --metrics --reasoning off --offline --host 127.0.0.1 --port 8080') {{ exit 2 }}; $p.CreationDate.ToUniversalTime().ToString('o')");
    let out = std::process::Command::new("powershell.exe")
        .args(["-NoProfile", "-NonInteractive", "-Command", &ps])
        .output()
        .map_err(|e| e.to_string())?;
    if !out.status.success() {
        return Err("operator-started server process/listener identity mismatch".into());
    }
    Ok(String::from_utf8(out.stdout)
        .map_err(|e| e.to_string())?
        .trim()
        .to_owned())
}
fn run(index: usize, evidence: &Path, pid: u32) -> Result<Value, String> {
    if !(1..=12).contains(&index) || pid == 0 {
        return Err("invalid arm ordinal or pid".into());
    }
    if next(evidence)?["next_ordinal"] != index {
        return Err("arm is not the next wholly unstarted arm".into());
    }
    if env::vars_os().any(|(k, _)| k.to_string_lossy().starts_with("LLAMA_ARG_")) {
        return Err("LLAMA_ARG_* must be absent".into());
    }
    if file_hash(Path::new(MODEL))? != MODEL_SHA || file_hash(Path::new(LLAMA))? != LLAMA_SHA {
        return Err("model or llama executable identity changed".into());
    }
    let creation = verify_server(pid)?;
    let (ci, ai) = ordinal(index - 1);
    let case_id = CASES[ci];
    let (arm_name, mode) = ARMS[ai];
    let root = root();
    let case = load_claim2_case(&root.join(format!(
        "fixtures/claim2/{}/case.json",
        case_id.to_lowercase()
    )))
    .map_err(|e| e.to_string())?;
    let (rows, _) = evidence_rows()?;
    let ledger = read_json(&root.join("fixtures/claim2/tokenization-request-ledger-v1.json"))?;
    let mut state = Claim2ArmState::new(&case, mode);
    let dir = evidence.join(format!("arm-{index:02}"));
    fs::create_dir(&dir).map_err(|e| e.to_string())?;
    write_new(&dir.join("START.json"),serde_json::to_vec_pretty(&json!({"ordinal":index,"case_id":case_id,"arm":arm_name,"server_pid":pid,"server_creation_utc":creation,"manifest_sha256":preflight()?["manifest_sha256"],"started_ms":now_ms()})).map_err(|e| e.to_string())?.as_slice())?;
    let client = Client::builder()
        .no_proxy()
        .redirect(Policy::none())
        .timeout(Duration::from_secs(1800))
        .build()
        .map_err(|e| e.to_string())?;
    let mut terminal = None;
    for slot in 1..=3 {
        let request = state.render_next(&case).map_err(|e| e.to_string())?;
        let id = format!("{case_id}/{arm_name}/slot-{slot}");
        check_body(&request.request_json, lookup(&rows, &id)?, &ledger, &id)?;
        let stem = format!("slot-{slot}");
        write_new(&dir.join(format!("{stem}-PLANNED.json")),serde_json::to_vec(&json!({"logical_request_id":id,"request_sha256":digest(&request.request_json),"server_pid":pid,"server_creation_utc":creation})).map_err(|e| e.to_string())?.as_slice())?;
        write_new(
            &dir.join(format!("{stem}-request.json")),
            &request.request_json,
        )?;
        write_new(&dir.join(format!("{stem}-DISPATCHING.json")),serde_json::to_vec(&json!({"logical_request_id":id,"request_sha256":digest(&request.request_json),"timestamp_ms":now_ms(),"server_pid":pid,"server_creation_utc":creation})).map_err(|e| e.to_string())?.as_slice())?;
        if verify_server(pid)? != creation {
            return Err("server instance changed before dispatch".into());
        }
        let response = client
            .post(ENDPOINT)
            .header("Content-Type", "application/json")
            .body(request.request_json)
            .send()
            .map_err(|e| format!("dispatch ambiguous; do not retry: {e}"))?;
        let status = response.status().as_u16();
        let bytes = response
            .bytes()
            .map_err(|e| format!("response ambiguous; do not retry: {e}"))?;
        write_new(&dir.join(format!("{stem}-raw-http-body.bin")), &bytes)?;
        write_new(&dir.join(format!("{stem}-OBSERVED.json")),serde_json::to_vec(&json!({"http_status":status,"raw_response_sha256":digest(&bytes),"raw_response_bytes":bytes.len(),"timestamp_ms":now_ms()})).map_err(|e| e.to_string())?.as_slice())?;
        if status != 200 {
            return Err(format!(
                "HTTP {status}; raw evidence retained; arm inconclusive"
            ));
        }
        let body: Value = serde_json::from_slice(&bytes)
            .map_err(|e| format!("invalid API envelope; raw retained: {e}"))?;
        let content = body["choices"][0]["message"]["content"]
            .as_str()
            .ok_or("assistant content missing; raw retained")?;
        let evaluation = state
            .record_output(&case, content.to_owned())
            .map_err(|e| e.to_string())?;
        let slot_record = &state.slots()[slot - 1];
        write_new(&dir.join(format!("{stem}-SCORED.json")),serde_json::to_vec_pretty(&json!({"logical_request_id":id,"assistant_content_sha256":digest(content.as_bytes()),"assistant_content":content,"usage":body["usage"],"finish_reason":body["choices"][0]["finish_reason"],"slot_status":slot_record.status,"evaluation":evaluation,"receipt_sha256":state.environment_receipts().last().map(|r|digest(r.text.as_bytes()))})).map_err(|e| e.to_string())?.as_slice())?;
        if slot_record.status == Claim2SlotStatus::Fail || evaluation.is_some() {
            terminal = evaluation;
            break;
        }
    }
    let success = terminal
        .as_ref()
        .map(|e| e.status == Claim2SlotStatus::Pass)
        .unwrap_or(false);
    let done = json!({"ordinal":index,"case_id":case_id,"arm":arm_name,"arm_task_success":success,"evaluation":terminal,"slots":state.slots(),"manifest_sha256":preflight()?["manifest_sha256"],"server_pid":pid,"server_creation_utc":creation,"completed_ms":now_ms()});
    write_new(
        &dir.join("DONE.json"),
        serde_json::to_vec_pretty(&done)
            .map_err(|e| e.to_string())?
            .as_slice(),
    )?;
    Ok(done)
}
fn main() {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let result = match args.as_slice() {
        [cmd] if cmd == "manifest" => manifest_bytes().and_then(|bytes| serde_json::from_slice(&bytes).map_err(|e| e.to_string())),
        [cmd] if cmd == "preflight" => preflight(),
        [cmd, flag, path] if cmd == "next" && flag == "--evidence" => next(Path::new(path)),
        [cmd, flag, path] if cmd == "report" && flag == "--evidence" => report(Path::new(path)),
        [cmd, o, n, e, path, s, pid] if cmd == "run" && o == "--ordinal" && e == "--evidence" && s == "--operator-started-pid" => {
            n.parse::<usize>().map_err(|e| e.to_string()).and_then(|n| pid.parse::<u32>().map_err(|e| e.to_string()).and_then(|pid| run(n,Path::new(path),pid)))
        }
        _ => Err("usage: preflight | next --evidence DIR | report --evidence DIR | run --ordinal N --evidence DIR --operator-started-pid PID".into()),
    };
    match result {
        Ok(value) => println!("{}", serde_json::to_string_pretty(&value).expect("JSON")),
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn arm(ok: bool, raw: &str) -> Value {
        json!({"arm_task_success":ok,"slots":[{"raw_assistant_output":raw}]})
    }
    #[test]
    fn frozen_positive_classifications() {
        assert_eq!(
            classify_arm_triplet("CP02", &[arm(false, "x"), arm(true, "x"), arm(true, "x")]),
            "BASELINE_INCAPABLE"
        );
        assert_eq!(
            classify_arm_triplet("CP02", &[arm(true, "x"), arm(false, "x"), arm(false, "x")]),
            "NO_OP_PIPELINE_REGRESSION"
        );
        assert_eq!(
            classify_arm_triplet("CP03", &[arm(true, "x"), arm(true, "x"), arm(false, "x")]),
            "DOWNSTREAM_SUCCESS_REGRESSED"
        );
        assert_eq!(
            classify_arm_triplet("CP03", &[arm(true, "x"), arm(true, "x"), arm(true, "x")]),
            "DOWNSTREAM_SUCCESS_PRESERVED"
        );
    }
    #[test]
    fn controls_compare_raw_outcomes() {
        assert_eq!(
            classify_arm_triplet("CP05", &[arm(true, "x"), arm(true, "x"), arm(true, "y")]),
            "CONTROL_OUTCOME_DIVERGENCE"
        );
        assert_eq!(
            classify_arm_triplet("CP06", &[arm(false, "x"), arm(false, "x"), arm(false, "x")]),
            "BASELINE_INCAPABLE"
        );
    }
    #[test]
    fn exact_body_rejects_altered_bytes() {
        let row = json!({"request_body_sha256":digest(b"a")});
        let ledger = json!({"requests":[{"logical_request_id":"x","request_body_sha256":digest(b"a"),"future_token_counter_body":"a"}]});
        assert!(check_body(b"a", &row, &ledger, "x").is_ok());
        assert!(check_body(b"a ", &row, &ledger, "x").is_err());
    }
}
