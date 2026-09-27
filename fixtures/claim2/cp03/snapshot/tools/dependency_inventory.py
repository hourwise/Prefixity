#!/usr/bin/env python3
"""Render the pinned dependency and build metadata in stable order."""

from __future__ import annotations

import argparse
import hashlib
import tomllib
from pathlib import Path


def version_tuple(value: str) -> tuple[int, int, int]:
    pieces = value.split(".")
    return tuple(int(piece) for piece in (pieces + ["0", "0"])[:3])  # type: ignore[return-value]


def caret_accepts(requirement: str, actual: str) -> bool:
    base = requirement.removeprefix("^")
    requested = version_tuple(base)
    found = version_tuple(actual)
    if found < requested or found[0] != requested[0]:
        return False
    if requested[0] == 0:
        if requested[1] == 0:
            return found[1:] == requested[1:]
        return found[1] == requested[1]
    return True


def same_caret_range(left: str, right: str) -> bool:
    return version_tuple(left.removeprefix("^")) == version_tuple(right.removeprefix("^"))


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", type=Path, required=True)
    parser.add_argument("--revision", required=True)
    args = parser.parse_args()
    root = args.root.resolve()
    input_files = sorted(
        (path for path in root.rglob("*") if path.is_file() and path.name != "SHA256SUMS"),
        key=lambda path: path.relative_to(root).as_posix(),
    )
    tree_hash = hashlib.sha256()
    for path in input_files:
        relative = path.relative_to(root).as_posix()
        tree_hash.update(relative.encode("utf-8"))
        tree_hash.update(b"\0")
        tree_hash.update(path.read_bytes())
        tree_hash.update(b"\0")

    workspace = tomllib.loads((root / "Cargo.toml").read_text(encoding="utf-8"))
    workspace_rust = workspace["workspace"]["package"]["rust-version"]
    members = workspace["workspace"]["members"]
    package_data: dict[str, dict[str, object]] = {}
    for member in members:
        manifest_path = root / member / "Cargo.toml"
        manifest = tomllib.loads(manifest_path.read_text(encoding="utf-8"))
        package = manifest["package"]
        dependencies = manifest.get("dependencies", {})
        features = manifest.get("features", {})
        targets = [
            target_name
            for target_name, target_path in (("lib", "src/lib.rs"), ("bin", "src/main.rs"))
            if (root / member / target_path).is_file()
        ]
        package_data[package["name"]] = {
            "version": package["version"],
            "manifest": manifest_path.relative_to(root).as_posix(),
            "dependencies": dependencies,
            "edition": workspace["workspace"]["package"]["edition"],
            "rust_version": workspace_rust,
            "features": features,
            "targets": targets,
        }

    protocol = package_data["wire-schema"]["version"]
    toolchain = tomllib.loads((root / "rust-toolchain.toml").read_text(encoding="utf-8"))["toolchain"]["channel"]
    ci_toolchains = [
        line.strip()
        for line in (root / ".ci/rust-toolchain-matrix.txt").read_text(encoding="utf-8").splitlines()
        if line.strip() and not line.startswith("#")
    ]
    edges = []
    for consumer in sorted(package_data):
        for dependency, declaration in sorted(package_data[consumer]["dependencies"].items()):
            dependency_name = declaration.get("package", dependency)
            requirement = declaration["version"]
            actual = package_data[dependency_name]["version"]
            edges.append((consumer, dependency_name, requirement, actual))
    outcomes = [caret_accepts(requirement, actual) for _, _, requirement, actual in edges]
    compatible = all(outcomes)
    toolchain_floor_present = workspace_rust + ".0" in ci_toolchains
    pinned_toolchain_present = toolchain in ci_toolchains
    compatibility = tomllib.loads((root / "compatibility.toml").read_text(encoding="utf-8"))
    lock = tomllib.loads((root / "Cargo.lock").read_text(encoding="utf-8"))
    locked_packages = {package["name"]: package["version"] for package in lock["package"]}
    release_profile = workspace.get("profile", {}).get("release", {})
    contract_results = []
    for consumer, details in compatibility["consumers"].items():
        requirement = details["requirement"]
        matching_edges = [
            edge for edge in edges
            if edge[0] == consumer and edge[1] == compatibility["protocol"]["package"]
        ]
        contract_results.append(
            bool(matching_edges)
            and all(same_caret_range(edge[2], requirement) for edge in matching_edges)
        )
    contracts_match = all(contract_results)

    print("project=dep-audit")
    print(f"snapshot_revision={args.revision}")
    print("inventory_producer=dependency-inventory-v1")
    print(f"snapshot_tree_sha256={tree_hash.hexdigest()}")
    print(f"workspace_rust_version={workspace_rust}")
    print(f"rust_toolchain_pin={toolchain}")
    print(f"ci_toolchains={','.join(ci_toolchains)}")
    print(f"release_profile_lto={str(release_profile.get('lto', False)).lower()}")
    print(f"release_profile_codegen_units={release_profile.get('codegen-units', 'default')}")
    print(f"release_profile_panic={release_profile.get('panic', 'default')}")
    print(f"protocol_contract={compatibility['protocol']['package']} {compatibility['protocol']['version']} wire_revision={compatibility['protocol']['wire_revision']}")
    print("workspace_members:")
    for member in members:
        package = tomllib.loads((root / member / "Cargo.toml").read_text(encoding="utf-8"))["package"]
        metadata = package_data[package["name"]]
        features = metadata["features"]
        feature_summary = ",".join(
            f"{name}=[{','.join(values)}]" for name, values in sorted(features.items())
        )
        targets = ",".join(metadata["targets"])
        locked = locked_packages.get(package["name"], "missing-lock-entry")
        print(
            f"  {member}: {package['name']} {package['version']} locked={locked} "
            f"edition={metadata['edition']} rust={metadata['rust_version']} "
            f"targets={targets} features={feature_summary}"
        )
    print("dependency_edges:")
    for (consumer, dependency, requirement, actual), accepted in zip(edges, outcomes):
        print(f"  {consumer} -> {dependency}: {requirement} resolves to {actual}; compatible={str(accepted).lower()}")
    print("protocol_consumer_contracts:")
    for consumer, details in sorted(compatibility["consumers"].items()):
        requirement = details["requirement"]
        matching_edges = [
            edge for edge in edges
            if edge[0] == consumer and edge[1] == compatibility["protocol"]["package"]
        ]
        declared = bool(matching_edges) and all(
            same_caret_range(edge[2], requirement) for edge in matching_edges
        )
        print(f"  {consumer}: requires {requirement}; manifest_matches={str(declared).lower()}")
    print(f"toolchain_floor_present_in_ci={str(toolchain_floor_present).lower()}")
    print(f"pinned_toolchain_present_in_ci={str(pinned_toolchain_present).lower()}")
    print(f"reconciliation={'compatible' if compatible and contracts_match and toolchain_floor_present and pinned_toolchain_present else 'incompatible'}")
    print(f"lock_package_count={len(locked_packages)}")
    print(f"wire_schema_version={protocol}")
    print(f"cache_engine_version={package_data['cache-engine']['version']}")
    print(f"audit_cli_version={package_data['audit-cli']['version']}")


if __name__ == "__main__":
    main()
