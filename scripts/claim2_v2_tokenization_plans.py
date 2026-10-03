"""Regenerate the two frozen Claim-2 V2 non-inference contact plans."""

import argparse
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
LEDGER = "fixtures/claim2/workload-request-ledger-v2.json"
MAP = "fixtures/claim2/token-evidence-inheritance-map-v2.json"
V1_LEDGER = "fixtures/claim2/tokenization-request-ledger-v1.json"
V1_RESULT = "docs/phase-1/PHASE_1C_CLAIM_2_TOKENIZATION_RESULT_V1.json"
PINS = {
    LEDGER: "a539c33355827ef574912ee72e235bf6301bd0acdc6666fa30effebed242a5eb",
    MAP: "2d33c1bd253f586d0363620ae14379ee2de49e58429598eff8b5b1c780bd268d",
    V1_LEDGER: "739205fb56e4f40bd55245f37d0768b8ca73c891b2f8d28bdb8f284e9f811d45",
    V1_RESULT: "727b577ce044d11c9b69cca33f7e3d3b23eab9785875d7445dda028b9a2b1377",
    "claim2-tokenization-pass-evidence-v1.json": "caf62ccaba1130a0e75d55316a1828cdcb17a8df4cc73f839f96f31a6110c264",
    "fixtures/claim2/tokenization-contact-plan-v1.json": "6b7634a3fc7a3d4b76289aea1772c1df686a6641309acc9e307e5f330dfefdf6",
}


def digest(data):
    return hashlib.sha256(data).hexdigest()


def pinned(path):
    data = (ROOT / path).read_bytes()
    allowed = {PINS[path]}
    if path == V1_RESULT:
        allowed.add("13a6cb66266ee64165650ca501b255b5d2643aee9b09c7166dd9e8031a8497be")
    assert digest(data) in allowed, f"pinned source changed: {path}"
    return json.loads(data)


def canonical_seal(path, expected):
    value = json.loads((ROOT / path).read_bytes())
    canonical = json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=False).encode()
    assert digest(canonical) == expected, f"canonical seal changed: {path}"
    sidecar = (ROOT / path).with_suffix(".sha256").read_text().split()[0]
    assert sidecar == expected, f"seal sidecar changed: {path}"


def build():
    ledger, inheritance, v1_ledger, v1_result = (pinned(p) for p in (LEDGER, MAP, V1_LEDGER, V1_RESULT))
    bindings = inheritance["v1_bindings"]
    canonical_seal(bindings["identity_path"], bindings["identity_canonical_sha256"])
    canonical_seal(V1_RESULT, bindings["interpreted_result_canonical_sha256"])
    assert bindings["tokenization_identity"] == "claim2-tokenization-v1-a5a6b896555db8296318f38010b7120dad8ad191e1329f030e0f738f31b90b91"
    assert bindings["terminal_classification"] == v1_result["terminal_classification"] == "WORKLOAD_TOKEN_ADMISSION_FAILED"
    assert bindings["request_ledger_sha256"] == PINS[V1_LEDGER]
    assert bindings["raw_evidence_sha256"] == PINS["claim2-tokenization-pass-evidence-v1.json"]
    assert bindings["contact_plan_sha256"] == PINS["fixtures/claim2/tokenization-contact-plan-v1.json"]

    v1_by_id = {r["logical_request_id"]: r for r in v1_ledger["requests"]}
    v1_counts = {r["request_body_sha256"]: r["input_tokens"] for r in v1_result["authoritative_unique_counts"]}
    map_by_hash = {r["request_body_sha256"]: r for r in inheritance["inherited_unique_request_hashes"]}
    assert len(map_by_hash) == 14
    assert len(ledger["requests"]) == 54
    assert ledger["cohort_order"] == ["CP02", "CP03", "CP07", "CP08", "CP05", "CP06"]
    groups = {}
    ids = set()
    for request in ledger["requests"]:
        case, arm, slot = request["case_id"], request["arm"], request["request_slot"]
        logical_id = f"{case}/{arm}/slot-{slot}"
        assert request["logical_request_id"] == logical_id and logical_id not in ids
        ids.add(logical_id)
        body = request["future_token_counter_body"].encode()
        body_json = json.loads(body)
        assert set(body_json) == {"model", "messages", "max_tokens", "temperature", "top_p", "seed", "stream"}
        assert body_json["messages"] == request["messages"]
        assert (body_json["max_tokens"], body_json["temperature"], body_json["top_p"], body_json["seed"], body_json["stream"]) == (1024, 0, 1, 1, False)
        assert request["request_body_sha256"] == digest(body)
        assert request["request_body_utf8_byte_length"] == len(body)
        assert request["tokenization_status"] == "NOT_PERFORMED" and not request["live_dispatchable"]
        hash_ = digest(body)
        group = groups.setdefault(hash_, {"body": body, "ids": [], "case": case, "slot": slot})
        assert group["body"] == body and group["case"] == case and group["slot"] == slot
        group["ids"].append(logical_id)
        if case in {"CP02", "CP03", "CP05", "CP06"}:
            prior = v1_by_id[logical_id]
            assert prior["future_token_counter_body"].encode() == body
            assert request["evidence_classification"] == "INHERITED_V1_TOKEN_COUNT"
            assert request["inherited_v1_input_tokens"] == v1_counts[hash_] == map_by_hash[hash_]["input_tokens"]
        else:
            assert case in {"CP07", "CP08"}
            assert request["evidence_classification"] == "NEW_TOKEN_COUNT_REQUIRED"
            assert request["inherited_v1_input_tokens"] is None
            assert hash_ not in v1_counts
    assert len(ids) == 54 and len(groups) == 22
    assert sum(len(g["ids"]) for h, g in groups.items() if h in map_by_hash) == 36
    assert sum(len(g["ids"]) for h, g in groups.items() if h not in map_by_hash) == 18
    for case in ["CP02", "CP03", "CP07", "CP08", "CP05", "CP06"]:
        for slot in (1, 2, 3):
            for arm in ("BASELINE", "NO_OP", "INTERVENTION"):
                assert f"{case}/{arm}/slot-{slot}" in ids
    for hash_, mapping in map_by_hash.items():
        assert sorted(groups[hash_]["ids"]) == sorted(mapping["v2_logical_request_ids"])
        assert len(groups[hash_]["body"]) == mapping["request_body_utf8_byte_length"]
    for case in ("CP07", "CP08"):
        per_case = [g for h, g in groups.items() if g["case"] == case]
        assert len(per_case) == 4
        assert sorted(len(g["ids"]) for g in per_case) == [1, 2, 3, 3]
    plans = {}
    for mode in ("HYBRID_8_NEW", "FULL_FRESH_22"):
        selected = [(h, g) for h, g in sorted(groups.items()) if mode == "FULL_FRESH_22" or h not in map_by_hash]
        entries = []
        for hash_, group in selected:
            group_ids = sorted(group["ids"])
            arms = {logical_id.split("/")[1] for logical_id in group_ids}
            equality = "ALL_ARMS" if len(arms) == 3 else "BASELINE_EQUALS_NO_OP" if arms == {"BASELINE", "NO_OP"} else "INTERVENTION_ONLY"
            entries.append({
                "request_body_sha256": hash_,
                "request_body_utf8_byte_length": len(group["body"]),
                "representative_logical_request_id": group_ids[0],
                "logical_request_ids": group_ids,
                "case_id": group["case"], "request_slot": group["slot"],
                "treatment_equality_class": equality,
                "exact_request_body": group["body"].decode(),
            })
        assert len(entries) == (8 if mode == "HYBRID_8_NEW" else 22)
        plans[mode] = {
            "schema_id": "prefixity.phase1c.claim2-tokenization-v2-contact-plan",
            "schema_version": 2,
            "status": "FROZEN_OFFLINE_NO_CONTACTS",
            "mode": mode,
            "source_ledger": {"path": LEDGER, "sha256": PINS[LEDGER]},
            "source_inheritance_map": {"path": MAP, "sha256": PINS[MAP]},
            "logical_request_count": 54, "total_unique_body_count": 22,
            "inherited_unique_body_count": 14 if mode == "HYBRID_8_NEW" else 0,
            "fresh_unique_body_count": len(entries),
            "maximum_readiness_contacts": 1,
            "maximum_token_count_contacts": len(entries),
            "inference_allowance": 0,
            "ordering": "request_body_sha256 ascending, lowercase ASCII hex",
            "endpoint_allowlist": {"readiness": {"method": "GET", "path": "/health"}, "token_count": {"method": "POST", "path": "/v1/chat/completions/input_tokens"}},
            "entries": entries,
        }
    return plans


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    for mode, plan in build().items():
        filename = "tokenization-contact-plan-v2-hybrid.json" if mode == "HYBRID_8_NEW" else "tokenization-contact-plan-v2-full-fresh.json"
        path = ROOT / "fixtures/claim2" / filename
        data = (json.dumps(plan, indent=2, ensure_ascii=False) + "\n").encode()
        if args.check:
            assert path.read_bytes() == data, f"frozen plan differs: {path}"
        else:
            path.write_bytes(data)
        print(mode, len(plan["entries"]), digest(data))


if __name__ == "__main__":
    main()
