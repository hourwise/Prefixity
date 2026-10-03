"""Seal the V2 token-only experiment after freezing a release executable."""

import argparse
import hashlib
import json
import subprocess
from pathlib import Path

from claim2_v2_tokenization_plans import ROOT, build

OUT = ROOT / "docs/phase-1/PHASE_1C_CLAIM_2_TOKENIZATION_IDENTITY_V2.json"
SOURCE_FILES = [
    "crates/prefixity-controlled-benchmark/Cargo.toml",
    "crates/prefixity-controlled-benchmark/src/lib.rs",
    "crates/prefixity-controlled-benchmark/src/phase1c_claim2_tokenization.rs",
    "crates/prefixity-controlled-benchmark/src/phase1c_claim2_v2_tokenization.rs",
    "crates/prefixity-controlled-benchmark/src/bin/phase1c_claim2_v2_tokenization.rs",
    "scripts/claim2_v2_tokenization_plans.py",
    "scripts/claim2_v2_tokenization_identity.py",
]
ARTIFACTS = {
    "v2_contract_sha256": "fixtures/claim2/workload-contract-v2.md",
    "v2_ledger_sha256": "fixtures/claim2/workload-request-ledger-v2.json",
    "v2_advancing_domain_sha256": "fixtures/claim2/advancing-output-domain-v2.json",
    "v2_inheritance_map_sha256": "fixtures/claim2/token-evidence-inheritance-map-v2.json",
    "v3_materialization_successor_sha256": "fixtures/claim2/materialization-report-v3.json",
}


def digest(data):
    return hashlib.sha256(data).hexdigest()


def git_bytes_at(commit, path):
    return subprocess.check_output(["git", "-c", "safe.directory=D:/Users/fleur/Prefixity", "show", f"{commit}:{path}"], cwd=ROOT)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--source-commit", required=True)
    parser.add_argument("--executable", type=Path, required=True)
    parser.add_argument("--windows-file-id", required=True)
    args = parser.parse_args()
    assert len(args.source_commit) == 40 and all(c in "0123456789abcdef" for c in args.source_commit)
    assert args.windows_file_id.startswith("0x")
    assert not OUT.exists(), "V2 identity already exists"
    plans = build()
    for mode, name in (("HYBRID_8_NEW", "tokenization-contact-plan-v2-hybrid.json"), ("FULL_FRESH_22", "tokenization-contact-plan-v2-full-fresh.json")):
        expected = (json.dumps(plans[mode], indent=2, ensure_ascii=False) + "\n").encode()
        assert (ROOT / "fixtures/claim2" / name).read_bytes() == expected
    source = {}
    for path in SOURCE_FILES:
        on_disk = (ROOT / path).read_bytes()
        recorded = git_bytes_at(args.source_commit, path)
        assert on_disk.replace(b"\r\n", b"\n") == recorded.replace(b"\r\n", b"\n"), f"source file drifted from commit: {path}"
        source[path] = digest(recorded)
    exe = args.executable.resolve(strict=True)
    exe_bytes = exe.read_bytes()
    ledger = json.loads((ROOT / ARTIFACTS["v2_ledger_sha256"]).read_bytes())
    inheritance = json.loads((ROOT / ARTIFACTS["v2_inheritance_map_sha256"]).read_bytes())
    bindings_v1 = inheritance["v1_bindings"]
    all_requests = [{
        "logical_request_id": r["logical_request_id"],
        "request_body_sha256": r["request_body_sha256"],
        "evidence_classification": r["evidence_classification"],
    } for r in ledger["requests"]]
    all_hashes = sorted({r["request_body_sha256"] for r in ledger["requests"]})
    inherited = sorted(inheritance["inherited_unique_request_hashes"], key=lambda item: item["request_body_sha256"])
    inherited_hashes = {r["request_body_sha256"] for r in inherited}
    new_hashes = sorted(set(all_hashes) - inherited_hashes)
    assert len(all_requests) == 54 and len(all_hashes) == 22 and len(inherited) == 14 and len(new_hashes) == 8
    assert sum(r["evidence_classification"] == "INHERITED_V1_TOKEN_COUNT" for r in ledger["requests"]) == 36
    assert sum(r["evidence_classification"] == "NEW_TOKEN_COUNT_REQUIRED" for r in ledger["requests"]) == 18
    bindings = {
        "materialization_commit": "98ef8a1edeb3097b6bdca264ea52fccf7cb14be6",
        "artifact_hashes": {key: digest((ROOT / path).read_bytes()) for key, path in ARTIFACTS.items()},
        "logical_request_identities": all_requests,
        "unique_request_hashes": all_hashes,
        "inherited_count_mappings": inherited,
        "v1_evidence_seals": {
            "experiment_id": bindings_v1["tokenization_identity"],
            "identity_seal": bindings_v1["identity_canonical_sha256"],
            "raw_evidence_sha256": bindings_v1["raw_evidence_sha256"],
            "interpreted_result_seal": bindings_v1["interpreted_result_canonical_sha256"],
            "request_ledger_sha256": bindings_v1["request_ledger_sha256"],
            "contact_plan_sha256": bindings_v1["contact_plan_sha256"],
        },
        "new_request_hashes": new_hashes,
        "plans": {
            "hybrid_path": "fixtures/claim2/tokenization-contact-plan-v2-hybrid.json",
            "hybrid_sha256": digest((ROOT / "fixtures/claim2/tokenization-contact-plan-v2-hybrid.json").read_bytes()),
            "hybrid_readiness_contacts": 1, "hybrid_input_token_contacts": 8,
            "full_fresh_path": "fixtures/claim2/tokenization-contact-plan-v2-full-fresh.json",
            "full_fresh_sha256": digest((ROOT / "fixtures/claim2/tokenization-contact-plan-v2-full-fresh.json").read_bytes()),
            "full_fresh_readiness_contacts": 1, "full_fresh_input_token_contacts": 22,
        },
        "instrument_identity": {
            "gguf_path": r"D:\Prefixity-Lab\models\Qwen3.5-9B\Qwen3.5-9B-Q4_K_M.gguf",
            "gguf_sha256": "cd76ec205963b3b33350093e6904d9de16c4e666fd104e1f632d25c7f15f2a13",
            "llama_path": r"C:\Users\USER\AppData\Local\Microsoft\WindowsApps\llama.exe",
            "llama_sha256": "cbe0655558e73168b3bc73f61aa70ec224475152b44c022f6e837616704d0617",
            "llama_build": "b10217-ddd4ec142",
        },
        "runtime_contract": {"context_tokens": 8192, "slots": 1, "reasoning": "off", "offline": True,
                             "host": "127.0.0.1", "port": 8080, "max_tokens": 1024,
                             "temperature": 0, "top_p": 1, "seed": 1, "stream": False,
                             "chat_template_kwargs": "ABSENT", "mmproj": "ABSENT", "hf": "ABSENT"},
        "client_source": {"source_commit": args.source_commit, "file_sha256": source},
        "client_executable": {"path": str(exe), "sha256": digest(exe_bytes), "bytes": len(exe_bytes),
                              "windows_file_id": args.windows_file_id},
        "zero_inference_rule": {"inference_allowance": 0,
                                "only_endpoints": [{"method": "GET", "path": "/health"},
                                                   {"method": "POST", "path": "/v1/chat/completions/input_tokens"}]},
        "plan_selection_rule": {"before_first_contact": True,
                                "hybrid_if": "all 14 V1 hashes, 36 mappings, seals, exact bytes, and accepted runtime/tokenizer identity verify",
                                "otherwise": "FULL_FRESH_22; no adaptive switch after contact",
                                "runtime_drift": "requires newly sealed instrument identity before contact"},
    }
    fingerprint = digest(json.dumps(bindings, sort_keys=True, separators=(",", ":"), ensure_ascii=False).encode())
    identity = {"schema_id": "prefixity.phase1c.claim2-tokenization-v2-identity", "schema_version": 2,
                "experiment_id": f"claim2-tokenization-v2-{fingerprint}",
                "classification": "OFFLINE_NON_INFERENCE_TOKENIZATION_PREPARATION",
                "bindings": bindings}
    OUT.write_bytes((json.dumps(identity, indent=2, ensure_ascii=False) + "\n").encode())
    seal = digest(json.dumps(identity, sort_keys=True, separators=(",", ":"), ensure_ascii=False).encode())
    OUT.with_suffix(".sha256").write_text(f"{seal}  {OUT.name} (canonical JSON: sorted keys, compact separators, UTF-8, no terminal newline)\n", encoding="utf-8", newline="\n")
    print(identity["experiment_id"], seal)


if __name__ == "__main__":
    main()
