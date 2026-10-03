"""Seal the offline Claim-2 V3 tokenization experiment from frozen artifacts.

This script is Windows-only because the accepted client has a Windows file ID.
It never opens a network connection or executes the client.
"""

from __future__ import annotations

import argparse
import ctypes
import hashlib
import json
import re
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
SOURCE_COMMIT = "b0cc0e66544b5428a6c67a4e876d6b7302031675"
MATERIALIZATION_COMMIT = "64d732dad2448e768ce12bc13a15bf3081f63205"
IDENTITY = ROOT / "docs/phase-1/PHASE_1C_CLAIM_2_TOKENIZATION_IDENTITY_V3.json"
SEAL = IDENTITY.with_suffix(".sha256")
FROZEN = (
    ROOT
    / "target/claim2-v3-tokenization-freeze"
    / SOURCE_COMMIT
    / "prefixity-phase1c-claim2-v3-tokenization.exe"
)
ARTIFACTS = {
    "v3_contract_sha256": ("fixtures/claim2/workload-contract-v3.md", "28d48448719433a9baa28c0b668cafade63c7858655be00a1fd9effbb638ac5e"),
    "v3_ledger_sha256": ("fixtures/claim2/workload-request-ledger-v3.json", "406586d71839d91bc565b9db5da69a3d4152193c4109f7876e6a1dc56d89dce2"),
    "v3_advancing_domain_sha256": ("fixtures/claim2/advancing-output-domain-v3.json", "d9f2de3056734026e364f0b2b89af8af8bf3cfbba78ded8e06533d9fc20c4186"),
    "v3_inheritance_map_sha256": ("fixtures/claim2/token-evidence-inheritance-map-v3.json", "6a53ea43c74d20d8c5bf99483ffdf79e6706656076014fd4eb7728198bcb7aa2"),
    "v4_materialization_successor_sha256": ("fixtures/claim2/materialization-report-v4.json", "dbf40e192c60b6f27b3e0112eb639be29a91e71b59d369b4f43ad682337c1e03"),
    "hybrid_sha256": ("fixtures/claim2/tokenization-contact-plan-v3-hybrid.json", "7897b44d1c6e3c43d387f760fe4994ff5ee94ca1d258e86a3bc2c91b6033ce7e"),
    "full_fresh_sha256": ("fixtures/claim2/tokenization-contact-plan-v3-full-fresh.json", "acb927d97df1a00f2e9a33d4fb7569ba1fc3eca89a0bd720042a28337b06fe5e"),
}
SOURCE_FILES = {
    "v3_module_sha256": "crates/prefixity-controlled-benchmark/src/phase1c_claim2_v3_tokenization.rs",
    "v3_binary_source_sha256": "crates/prefixity-controlled-benchmark/src/bin/phase1c_claim2_v3_tokenization.rs",
    "v3_renderer_sha256": "crates/prefixity-controlled-benchmark/examples/support/claim2_v3_workload.rs",
}


def digest(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def canonical(value: object) -> bytes:
    return json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=False).encode("utf-8")


def file_id(path: Path) -> str:
    result = subprocess.run(
        ["fsutil", "file", "queryfileid", str(path)],
        check=True,
        capture_output=True,
        text=True,
    )
    match = re.search(r"0x[0-9a-fA-F]{32}", result.stdout)
    if match is None:
        raise ValueError("Windows file ID was not returned for frozen client")
    return match.group(0).lower()


def volume_index_file_id(path: Path) -> str:
    class FileTime(ctypes.Structure):
        _fields_ = [("low", ctypes.c_uint32), ("high", ctypes.c_uint32)]

    class ByHandleFileInformation(ctypes.Structure):
        _fields_ = [
            ("attributes", ctypes.c_uint32),
            ("creation", FileTime),
            ("access", FileTime),
            ("write", FileTime),
            ("volume", ctypes.c_uint32),
            ("size_high", ctypes.c_uint32),
            ("size_low", ctypes.c_uint32),
            ("links", ctypes.c_uint32),
            ("index_high", ctypes.c_uint32),
            ("index_low", ctypes.c_uint32),
        ]

    kernel32 = ctypes.WinDLL("kernel32", use_last_error=True)
    create = kernel32.CreateFileW
    create.argtypes = [ctypes.c_wchar_p, ctypes.c_uint32, ctypes.c_uint32,
                       ctypes.c_void_p, ctypes.c_uint32, ctypes.c_uint32,
                       ctypes.c_void_p]
    create.restype = ctypes.c_void_p
    get_info = kernel32.GetFileInformationByHandle
    get_info.argtypes = [ctypes.c_void_p, ctypes.POINTER(ByHandleFileInformation)]
    get_info.restype = ctypes.c_int
    close = kernel32.CloseHandle
    close.argtypes = [ctypes.c_void_p]
    close.restype = ctypes.c_int
    handle = create(str(path), 0x80, 7, None, 3, 0x80, None)
    if handle == ctypes.c_void_p(-1).value:
        raise OSError(ctypes.get_last_error(), "CreateFileW failed")
    try:
        info = ByHandleFileInformation()
        if not get_info(handle, ctypes.byref(info)):
            raise OSError(ctypes.get_last_error(), "GetFileInformationByHandle failed")
        return f"volume={info.volume:08x};index={info.index_high:08x}{info.index_low:08x}"
    finally:
        close(handle)


def build() -> dict:
    head = subprocess.run(
        ["git", "-c", "safe.directory=D:/Users/fleur/Prefixity", "rev-parse", "HEAD"],
        cwd=ROOT,
        check=True,
        capture_output=True,
        text=True,
    ).stdout.strip()
    if head != SOURCE_COMMIT:
        # The final documentation commit is permitted after provenance freeze.
        ancestor = subprocess.run(
            ["git", "-c", "safe.directory=D:/Users/fleur/Prefixity", "merge-base", "--is-ancestor", SOURCE_COMMIT, "HEAD"],
            cwd=ROOT,
            check=False,
        )
        if ancestor.returncode != 0:
            raise ValueError("source-provenance commit is not an ancestor of HEAD")
    artifacts = {}
    for key, (relative, expected) in ARTIFACTS.items():
        actual = digest((ROOT / relative).read_bytes())
        if actual != expected:
            raise ValueError(f"frozen artifact hash changed: {relative}")
        artifacts[key] = actual
    ledger = json.loads((ROOT / ARTIFACTS["v3_ledger_sha256"][0]).read_bytes())
    inheritance = json.loads((ROOT / ARTIFACTS["v3_inheritance_map_sha256"][0]).read_bytes())
    hybrid = json.loads((ROOT / ARTIFACTS["hybrid_sha256"][0]).read_bytes())
    full = json.loads((ROOT / ARTIFACTS["full_fresh_sha256"][0]).read_bytes())
    requests = ledger["requests"]
    logical = [
        {
            "logical_request_id": row["logical_request_id"],
            "request_body_sha256": row["request_body_sha256"],
            "evidence_classification": row["evidence_classification"],
        }
        for row in requests
    ]
    unique = [entry["request_body_sha256"] for entry in full["entries"]]
    new = [entry["request_body_sha256"] for entry in hybrid["entries"]]
    inherited = [
        {
            "request_body_sha256": entry["request_body_sha256"],
            "input_tokens": entry["input_tokens"],
            "v3_logical_request_ids": entry["v3_logical_request_ids"],
        }
        for entry in inheritance["inherited_unique_request_hashes"]
    ]
    if not (len(logical) == 54 and len(set(unique)) == 22 and len(inherited) == 14 and len(new) == 8):
        raise ValueError("V3 request population changed")
    if len([row for row in requests if row["evidence_classification"] == "INHERITED_V1_TOKEN_COUNT"]) != 36:
        raise ValueError("V1 inherited logical coverage changed")
    if len([row for row in requests if row["evidence_classification"] == "NEW_TOKEN_COUNT_REQUIRED"]) != 18:
        raise ValueError("V3 fresh logical coverage changed")
    source = {
        key: digest((ROOT / relative).read_bytes().replace(b"\r\n", b"\n"))
        for key, relative in SOURCE_FILES.items()
    }
    executable = {
        "path": str(FROZEN),
        "sha256": digest(FROZEN.read_bytes()),
        "bytes": FROZEN.stat().st_size,
        "windows_file_id": file_id(FROZEN),
        "volume_index_file_id": volume_index_file_id(FROZEN),
    }
    bindings = {
        "materialization_commit": MATERIALIZATION_COMMIT,
        "artifact_hashes": {key: artifacts[key] for key in ARTIFACTS if key not in ("hybrid_sha256", "full_fresh_sha256")},
        "logical_request_identities": logical,
        "unique_request_body_hashes": unique,
        "new_request_body_hashes": new,
        "inherited_count_mappings": inherited,
        "v1_seals": {
            "identity": "4b7265e8a509829b3109947302ec82c2efec4f05cedd3aeac926324fa2a11f16",
            "raw": "caf62ccaba1130a0e75d55316a1828cdcb17a8df4cc73f839f96f31a6110c264",
            "result": "03263b532a13619f9e6e38a447bf984651a712944d7c9aa83df9a86764821040",
        },
        "plans": {"hybrid_sha256": artifacts["hybrid_sha256"], "full_fresh_sha256": artifacts["full_fresh_sha256"]},
        "instrument_identity": {
            "gguf_path": r"D:\Prefixity-Lab\models\Qwen3.5-9B\Qwen3.5-9B-Q4_K_M.gguf",
            "gguf_sha256": "cd76ec205963b3b33350093e6904d9de16c4e666fd104e1f632d25c7f15f2a13",
            "llama_path": r"C:\Users\USER\AppData\Local\Microsoft\WindowsApps\llama.exe",
            "llama_sha256": "cbe0655558e73168b3bc73f61aa70ec224475152b44c022f6e837616704d0617",
            "llama_build": "b10217-ddd4ec142",
            "runtime_contract": {
                "context_tokens": 8192, "slots": 1, "reasoning": "off", "offline": True,
                "host": "127.0.0.1", "port": 8080, "max_tokens": 1024,
                "temperature": 0, "top_p": 1, "seed": 1, "stream": False,
                "chat_template_kwargs": "ABSENT",
            },
            "endpoint_allowlist": {
                "readiness": {"method": "GET", "path": "/health"},
                "token_count": {"method": "POST", "path": "/v1/chat/completions/input_tokens"},
            },
        },
        "source_provenance": {"commit": SOURCE_COMMIT, **source},
        "client_executable": executable,
        "zero_inference_rule": {"inference_allowance": 0},
        "plan_selection_rule": {
            "before_first_contact": True,
            "immutable_after_first_contact": True,
            "eligible_inheritance": "HYBRID_8_NEW",
            "unverifiable_inheritance_with_matching_instrument": "FULL_FRESH_22",
            "instrument_drift": "ABORT_AND_REFREEZE",
        },
    }
    experiment_id = "claim2-tokenization-v3-" + digest(canonical(bindings))
    return {
        "schema_id": "prefixity.phase1c.claim2-tokenization-v3-identity",
        "schema_version": 3,
        "experiment_id": experiment_id,
        "classification": "OFFLINE_NON_INFERENCE_TOKENIZATION_PREPARATION",
        "bindings": bindings,
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--write", action="store_true")
    args = parser.parse_args()
    identity = build()
    data = (json.dumps(identity, indent=2, ensure_ascii=False) + "\n").encode("utf-8")
    seal = digest(canonical(identity))
    sidecar = (seal + "  " + IDENTITY.name + "\n").encode("ascii")
    if args.write:
        IDENTITY.write_bytes(data)
        SEAL.write_bytes(sidecar)
    elif IDENTITY.read_bytes() != data or SEAL.read_bytes() != sidecar:
        raise ValueError("V3 identity or canonical sidecar differs from regeneration")
    print(identity["experiment_id"])
    print(seal)


if __name__ == "__main__":
    main()
