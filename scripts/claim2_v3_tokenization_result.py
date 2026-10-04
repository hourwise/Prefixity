"""Reproduce the sealed Claim-2 V3 hybrid tokenization interpretation offline."""

import argparse
import hashlib
import json
from collections import defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
RAW = "claim2-v3-hybrid-tokenization-pass-evidence.json"
OUT = "docs/phase-1/PHASE_1C_CLAIM_2_TOKENIZATION_RESULT_V3.json"
IDENTITY = "docs/phase-1/PHASE_1C_CLAIM_2_TOKENIZATION_IDENTITY_V3.json"
LEDGER = "fixtures/claim2/workload-request-ledger-v3.json"
PLAN = "fixtures/claim2/tokenization-contact-plan-v3-hybrid.json"
MAP = "fixtures/claim2/token-evidence-inheritance-map-v3.json"
V1_RESULT = "docs/phase-1/PHASE_1C_CLAIM_2_TOKENIZATION_RESULT_V1.json"
PINS = {
    RAW: "b081e435982e9354d0b72a16964c0f5e62b2b5beb25383be0553b9647b21ee83",
    LEDGER: "406586d71839d91bc565b9db5da69a3d4152193c4109f7876e6a1dc56d89dce2",
    PLAN: "7897b44d1c6e3c43d387f760fe4994ff5ee94ca1d258e86a3bc2c91b6033ce7e",
    MAP: "6a53ea43c74d20d8c5bf99483ffdf79e6706656076014fd4eb7728198bcb7aa2",
}
EXPERIMENT = "claim2-tokenization-v3-570cc81cbc7ddbba5c3e3128571f1197b79704b0e7af3868e6cecc1db443960f"
IDENTITY_SEAL = "6dfca31b408b780e5aa318fdd1caa896bda6f5f91b72176555edc69255e51d01"
V1_RESULT_SEAL = "03263b532a13619f9e6e38a447bf984651a712944d7c9aa83df9a86764821040"
SERVER = {
    "pid": 20260, "parent_pid": 2356,
    "creation_time_local": "2026-10-04T08:39:11",
    "executable_path": r"C:\Users\USER\AppData\Local\Microsoft\WindowsApps\llama.exe",
    "command_line": r'"C:\Users\USER\AppData\Local\Microsoft\WindowsApps\llama.exe" serve -m D:\Prefixity-Lab\models\Qwen3.5-9B\Qwen3.5-9B-Q4_K_M.gguf -c 8192 -np 1 --metrics --reasoning off --offline --host 127.0.0.1 --port 8080',
    "listener": {"local_address": "127.0.0.1", "local_port": 8080, "owning_pid": 20260},
    "matching_server_processes": 1, "matching_listeners": 1,
}


def sha(data):
    return hashlib.sha256(data).hexdigest()


def canonical(value):
    return json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=False).encode()


def pinned(path):
    data = (ROOT / path).read_bytes()
    assert sha(data) == PINS[path], f"pinned bytes changed: {path}"
    return json.loads(data)


def sealed(path, seal):
    value = json.loads((ROOT / path).read_bytes())
    assert sha(canonical(value)) == seal
    assert (ROOT / path).with_suffix(".sha256").read_text().split()[0] == seal
    return value


def validate_sources():
    identity = sealed(IDENTITY, IDENTITY_SEAL)
    assert identity["experiment_id"] == EXPERIMENT
    assert identity["bindings"]["materialization_commit"] == "64d732dad2448e768ce12bc13a15bf3081f63205"
    assert identity["bindings"]["plans"]["hybrid_sha256"] == PINS[PLAN]
    assert identity["bindings"]["artifact_hashes"]["v3_ledger_sha256"] == PINS[LEDGER]
    assert identity["bindings"]["artifact_hashes"]["v3_inheritance_map_sha256"] == PINS[MAP]
    raw, ledger, plan, mapping = (pinned(p) for p in (RAW, LEDGER, PLAN, MAP))
    v1 = sealed(V1_RESULT, V1_RESULT_SEAL)
    assert v1["terminal_classification"] == "WORKLOAD_TOKEN_ADMISSION_FAILED"
    assert identity["bindings"]["v1_seals"]["result"] == V1_RESULT_SEAL
    return identity, raw, ledger, plan, mapping, v1


def validate_raw(raw, plan, identity, mapping):
    assert raw["schema_id"] == "prefixity.phase1c.claim2-tokenization-v3-pass-evidence"
    assert raw["schema_version"] == 3
    assert raw["status"] == "TOKENIZATION_COUNTS_COMPLETE_PENDING_REVIEW"
    assert raw["terminal_classification"] is None
    assert raw["source_identities"] == {"ledger_sha256": PINS[LEDGER], "contact_plan_sha256": PINS[PLAN]}
    assert raw["planned_accounting"] == {
        "logical_requests": 54, "unique_request_bodies": 22,
        "duplicate_logical_requests_avoided": 32,
        "maximum_readiness_contacts": 1, "maximum_token_count_contacts": 8,
        "inference_allowance": 0}
    assert raw["contact_accounting"] == {
        "readiness_contacts": 1, "unique_token_count_contacts": 8, "inference_requests": 0}
    assert raw["readiness"]["status"] == "READY" and raw["readiness"]["http_status"] == 200
    assert raw["readiness"]["error"] is None
    runtime = raw["operator_runtime_confirmation"]
    assert runtime["selected_mode"] == "HYBRID_8_NEW"
    assert runtime["experiment_identity"] == EXPERIMENT
    assert runtime["operator_started_server"] is True
    assert runtime["model_sha256"] == identity["bindings"]["instrument_identity"]["gguf_sha256"]
    assert runtime["llama_sha256"] == identity["bindings"]["instrument_identity"]["llama_sha256"]
    assert (runtime["reasoning"], runtime["context_tokens"], runtime["slots"],
            runtime["offline"], runtime["host"], runtime["port"]) == (
            "off", "8192", "1", True, "127.0.0.1", "8080")
    assert len(runtime["inherited_count_mappings"]) == len(mapping["inherited_unique_request_hashes"]) == 14
    assert runtime["inherited_count_mappings"] == identity["bindings"]["inherited_count_mappings"]
    for sealed, complete in zip(runtime["inherited_count_mappings"], mapping["inherited_unique_request_hashes"]):
        assert all(complete[key] == value for key, value in sealed.items())
    assert plan["mode"] == "HYBRID_8_NEW" and plan["inference_allowance"] == 0
    assert len(raw["unique_request_results"]) == len(plan["entries"]) == 8
    fresh = {}
    for actual, entry in zip(raw["unique_request_results"], plan["entries"]):
        h = entry["request_body_sha256"]
        assert actual["request_body_sha256"] == h
        assert actual["representative_logical_request_id"] == entry["representative_logical_request_id"]
        assert actual["logical_request_ids"] == entry["logical_request_ids"]
        assert actual["status"] == "COUNTED" and actual["http_status"] == 200
        assert actual["response_count_field"] == "input_tokens" and actual["error"] is None
        assert isinstance(actual["input_tokens"], int) and actual["input_tokens"] >= 0
        assert len(actual["response_body_sha256"]) == 64
        assert actual["response_body_utf8_byte_length"] <= 65536
        assert h not in fresh
        fresh[h] = actual
    return fresh


def map_counts(ledger, mapping, v1, fresh, identity):
    inherited = {x["request_body_sha256"]: x for x in mapping["inherited_unique_request_hashes"]}
    v1_counts = {x["request_body_sha256"]: x["input_tokens"] for x in v1["authoritative_unique_counts"]}
    assert len(inherited) == 14 and len(v1_counts) == 22 and set(inherited).isdisjoint(fresh)
    for h, record in inherited.items():
        assert v1_counts[h] == record["input_tokens"]
    logical, groups, seen = [], defaultdict(list), set()
    for request in ledger["requests"]:
        logical_id = request["logical_request_id"]
        body = request["future_token_counter_body"].encode()
        h = sha(body)
        assert logical_id not in seen
        seen.add(logical_id)
        assert h == request["request_body_sha256"]
        assert len(body) == request["request_body_utf8_byte_length"]
        assert json.loads(body)["messages"] == request["messages"]
        if h in inherited:
            record = inherited[h]
            count, origin = record["input_tokens"], "INHERITED_V1_TOKEN_COUNT"
            assert logical_id in record["v3_logical_request_ids"]
            assert request["inherited_v1_input_tokens"] == count
        else:
            record = fresh[h]
            count, origin = record["input_tokens"], "FRESH_V3_INPUT_TOKEN_COUNT"
            assert logical_id in record["logical_request_ids"]
            assert request["inherited_v1_input_tokens"] is None
        assert request["evidence_classification"] == (
            "INHERITED_V1_TOKEN_COUNT" if h in inherited else "NEW_TOKEN_COUNT_REQUIRED")
        groups[h].append(logical_id)
        logical.append({"logical_request_id": logical_id, "case_id": request["case_id"],
                        "arm": request["arm"], "request_slot": request["request_slot"],
                        "request_body_sha256": h, "input_tokens": count,
                        "measurement_origin": origin})
    assert len(logical) == len(seen) == 54 and len(groups) == 22
    assert sum(x["measurement_origin"] == "INHERITED_V1_TOKEN_COUNT" for x in logical) == 36
    assert sum(x["measurement_origin"] == "FRESH_V3_INPUT_TOKEN_COUNT" for x in logical) == 18
    assert sorted(groups) == identity["bindings"]["unique_request_body_hashes"]
    assert [(x["logical_request_id"], x["request_body_sha256"]) for x in logical] == [
        (x["logical_request_id"], x["request_body_sha256"]) for x in identity["bindings"]["logical_request_identities"]]
    unique = []
    for h, ids in sorted(groups.items()):
        if h in inherited:
            count, origin = inherited[h]["input_tokens"], "INHERITED_V1_TOKEN_COUNT"
            source = {"v1_result_seal": V1_RESULT_SEAL}
        else:
            observed = fresh[h]
            count, origin = observed["input_tokens"], "FRESH_V3_INPUT_TOKEN_COUNT"
            source = {"http_status": 200, "response_body_sha256": observed["response_body_sha256"],
                      "response_body_utf8_byte_length": observed["response_body_utf8_byte_length"]}
        unique.append({"request_body_sha256": h, "logical_request_ids": sorted(ids),
                       "input_tokens": count, "measurement_origin": origin, "source": source})
    return logical, unique

def evaluate_cases(ledger, logical):
    by_id = {x["logical_request_id"]: x["input_tokens"] for x in logical}
    cases = []
    for case in ledger["cohort_order"]:
        arms = {arm: [by_id[f"{case}/{arm}/slot-{slot}"] for slot in (1, 2, 3)]
                for arm in ("BASELINE", "NO_OP", "INTERVENTION")}
        assert arms["BASELINE"] == arms["NO_OP"]
        records = [x for x in logical if x["case_id"] == case]
        context_pass = all(x["input_tokens"] <= 6000 and x["input_tokens"] + 1024 <= 8192
                           for x in records)
        if case in ("CP05", "CP06"):
            zero = arms["BASELINE"] == arms["INTERVENTION"]
            assert zero
            metrics = {"control_zero_mutation": zero, "legal_reduction_tokens": 0}
            reduction_pass = zero
        else:
            b, i = arms["BASELINE"], arms["INTERVENTION"]
            d3, bsum, isum = b[2] - i[2], sum(b), sum(i)
            dsum = bsum - isum
            gates = {"D3_at_least_800": d3 >= 800,
                     "R3_at_least_0_20": 5 * d3 >= b[2],
                     "Rsum_at_least_0_08": 25 * dsum >= 2 * bsum}
            metrics = {"D3": d3, "Bsum": bsum, "Isum": isum, "Dsum": dsum,
                       "R3": d3 / b[2], "Rsum": dsum / bsum, "gates": gates}
            reduction_pass = all(gates.values())
        cases.append({"case_id": case, "arms_input_tokens_by_slot": arms, "metrics": metrics,
                      "reduction_or_control_pass": reduction_pass,
                      "context_pass": context_pass,
                      "admission_pass": reduction_pass and context_pass})
    return cases


def context_result(logical):
    above_input = [x["logical_request_id"] for x in logical if x["input_tokens"] > 6000]
    above_reserve = [x["logical_request_id"] for x in logical if x["input_tokens"] + 1024 > 8192]
    maximum = max(x["input_tokens"] for x in logical)
    max_ids = [x["logical_request_id"] for x in logical if x["input_tokens"] == maximum]
    assert maximum == 7569
    assert max_ids == ["CP09/BASELINE/slot-3", "CP09/NO_OP/slot-3"]
    return {"input_limit": 6000, "context_tokens": 8192, "output_reserve_tokens": 1024,
            "maximum_input_tokens": maximum, "maximum_logical_request_ids": max_ids,
            "maximum_with_output_reserve": maximum + 1024,
            "above_6000_logical_request_ids": above_input,
            "above_8192_with_reserve_logical_request_ids": above_reserve,
            "all_requests_pass": not above_input and not above_reserve}


def cpu_result(logical):
    totals = {arm: sum(x["input_tokens"] for x in logical if x["arm"] == arm)
              for arm in ("BASELINE", "NO_OP", "INTERVENTION")}
    total = sum(totals.values())
    round6 = lambda x: round(x, 6)
    scenarios = []
    for output_per_request in (50, 80, 150, 1024):
        completion_total = len(logical) * output_per_request
        scenarios.append({
            "assumed_completion_tokens_per_request": output_per_request,
            "assumed_completion_tokens_total": completion_total,
            "decode_hours_at_2_to_1_2_tokens_per_second":
                [round6(completion_total / rate / 3600) for rate in (2, 1.2)],
            "input_plus_decode_hours_before_startups_at_fast_to_slow_rates": [
                round6(total / 10 / 3600 + completion_total / 2 / 3600),
                round6(total / 9 / 3600 + completion_total / 1.2 / 3600)]})
    return {"classification": "CPU_RUNTIME_PRACTICALITY_REVIEW_REQUIRED",
            "historical_model_capability_classification": "MODEL_CAPABILITY_ACCEPTED",
            "logical_input_tokens_by_arm": totals,
            "complete_cohort_logical_input_tokens": total,
            "fresh_server_starts_for_18_arms": 18,
            "fresh_start_duration_measured": False,
            "planning_prefill_rate_tokens_per_second": [9, 10],
            "planning_prefill_hours_at_10_to_9_tokens_per_second":
                [round6(total / rate / 3600) for rate in (10, 9)],
            "planning_decode_rate_tokens_per_second": [1.2, 2],
            "output_scenarios": scenarios,
            "interpretation": "Historical rounded-rate arithmetic only; V3 runtime was not measured and does not alter token admission."}


def build():
    identity, raw, ledger, plan, mapping, v1 = validate_sources()
    fresh = validate_raw(raw, plan, identity, mapping)
    logical, unique = map_counts(ledger, mapping, v1, fresh, identity)
    cases = evaluate_cases(ledger, logical)
    context = context_result(logical)
    cpu = cpu_result(logical)
    terminal = ("WORKLOAD_V3_TOKEN_ADMISSION_PASSED" if context["all_requests_pass"]
                and all(case["admission_pass"] for case in cases)
                else "WORKLOAD_V3_TOKEN_ADMISSION_FAILED")
    assert terminal == "WORKLOAD_V3_TOKEN_ADMISSION_FAILED"
    return {
        "schema_id": "prefixity.phase1c.claim2-tokenization-v3-interpreted-result",
        "schema_version": 3,
        "terminal_classification": terminal,
        "selected_mode": "HYBRID_8_NEW",
        "provenance": {
            "promoted_preparation_commit": "3653035dbb85548569fd124f3feb3f0efae721de",
            "source_provenance_commit": "b0cc0e66544b5428a6c67a4e876d6b7302031675",
            "experiment_id": EXPERIMENT, "experiment_identity_seal": IDENTITY_SEAL,
            "v1_result_seal": V1_RESULT_SEAL,
            "v1_identity_seal": identity["bindings"]["v1_seals"]["identity"],
            "v1_raw_evidence_sha256": identity["bindings"]["v1_seals"]["raw"],
            "ledger_sha256": PINS[LEDGER], "hybrid_plan_sha256": PINS[PLAN],
            "raw_evidence_path": RAW, "raw_evidence_sha256": PINS[RAW],
            "raw_evidence_bytes": (ROOT / RAW).stat().st_size,
            "frozen_client": identity["bindings"]["client_executable"],
            "gguf_sha256": identity["bindings"]["instrument_identity"]["gguf_sha256"],
            "llama_sha256": identity["bindings"]["instrument_identity"]["llama_sha256"],
            "precontact_server": SERVER,
        },
        "contact_accounting": raw["contact_accounting"],
        "readiness": raw["readiness"],
        "fresh_authoritative_counts": [
            x for x in unique if x["measurement_origin"] == "FRESH_V3_INPUT_TOKEN_COUNT"],
        "authoritative_unique_counts": unique,
        "authoritative_logical_counts": logical,
        "case_results": cases,
        "context_fit": context,
        "cpu_practicality": cpu,
        "failed_admission_gates": {
            "context_input_over_6000": context["above_6000_logical_request_ids"],
            "context_with_reserve_over_8192": context["above_8192_with_reserve_logical_request_ids"],
            "positive_or_control_failures": [
                x["case_id"] for x in cases if not x["reduction_or_control_pass"]]},
        "interpretation_boundary": {
            "no_inference": True, "no_scored_pilot": True,
            "workload_or_threshold_modified": False,
            "server_shutdown_sealed_separately": True},
    }


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--write", action="store_true")
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    assert args.write != args.check, "choose exactly one of --write or --check"
    result = build()
    path = ROOT / OUT
    rendered = (json.dumps(result, indent=2, ensure_ascii=False, sort_keys=True) + "\n").encode()
    seal = sha(canonical(result))
    sidecar = (f"{seal}  {path.name} "
               "(canonical JSON: sorted keys, compact separators, UTF-8, no terminal newline)\n").encode()
    if args.write:
        assert not path.exists() and not path.with_suffix(".sha256").exists()
        path.write_bytes(rendered)
        path.with_suffix(".sha256").write_bytes(sidecar)
    else:
        assert path.read_bytes() == rendered
        assert path.with_suffix(".sha256").read_bytes() == sidecar
    print(result["terminal_classification"], seal,
          result["context_fit"]["maximum_input_tokens"],
          result["cpu_practicality"]["complete_cohort_logical_input_tokens"])


if __name__ == "__main__":
    main()
