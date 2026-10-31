#!/usr/bin/env python3
# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0
"""Validate the canonical schema inventory with the standard protobuf compiler."""

import argparse
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import shlex
import subprocess
import sys
import tempfile


ROOT = Path(__file__).resolve().parents[1]


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def require(condition, message):
    if not condition:
        raise ValueError(message)


def validate(root, protobuf_include, protoc="protoc"):
    manifest = json.loads((root / "schemas/manifest.json").read_text())
    entries = manifest["schemas"]
    paths = [entry["path"] for entry in entries]
    require(paths == sorted(set(paths)), "schema inventory must be sorted and unique")
    for entry in entries:
        path = PurePosixPath(entry["path"])
        require(path.parts[:3] == ("schemas", "gz", "msgs")
                and ".." not in path.parts and path.suffix == ".proto",
                "invalid canonical schema path")
        source = root / path
        require(source.is_file() and not source.is_symlink(), f"missing regular schema: {path}")
        require(digest(source) == entry["sha256"], f"schema hash mismatch: {path}")
    actual = {path.relative_to(root).as_posix() for path in root.rglob("*.proto")
              if "artifacts" not in path.relative_to(root).parts}
    require(actual == set(paths), "unexpected or missing .proto inventory")
    require(not (root / "Cargo.toml").exists() and not (root / "src").exists()
            and not (root / "vendor").exists(), "schema release must not contain a Rust runtime")
    policy = manifest["validation"]
    version = subprocess.check_output([protoc, "--version"], text=True).strip()
    require(version == policy["protoc_version"], f"unexpected protoc version: {version}")
    external = []
    for entry in policy["external_imports"]:
        path = PurePosixPath(entry["path"])
        require(not path.is_absolute() and ".." not in path.parts,
                "invalid external import path")
        source = protobuf_include / path
        require(source.is_file() and digest(source) == entry["sha256"],
                f"external import hash mismatch: {path}")
        external.append(source.resolve())
    # A single public-import root lets protoc report its actual complete closure.
    # No custom protobuf grammar or descriptor decoder is needed here.
    with tempfile.TemporaryDirectory(prefix="gz-schema-validation-") as directory:
        temporary = Path(directory)
        catalog = temporary / "schema_validation.proto"
        catalog.write_text('syntax = "proto3";\n' + "".join(
            f'import public "{path.removeprefix("schemas/")}";\n' for path in paths))
        command = [protoc, f"-I{temporary}", f"-I{root / 'schemas'}",
                   f"-I{protobuf_include}", "--include_imports",
                   "--descriptor_set_out=descriptor.pb", "--dependency_out=dependencies.d",
                   catalog.name]
        result = subprocess.run(command, cwd=temporary, capture_output=True, text=True, check=True)
        require(not result.stderr, f"protoc diagnostics: {result.stderr}")
        depfile = (temporary / "dependencies.d").read_text()
        require(depfile.startswith("descriptor.pb:"), "unexpected protoc dependency output")
        dependencies = {Path(path).resolve() for path in shlex.split(
            depfile.removeprefix("descriptor.pb:").replace("\\\n", ""))}
        expected = {root.joinpath(path).resolve() for path in paths} | set(external) | {catalog}
        require(dependencies == expected, "compiler-resolved import closure differs from manifest")
        descriptor = digest(temporary / "descriptor.pb")
        require(descriptor == policy["descriptor_sha256"], "compiled descriptor identity differs")
    return {"schema_count": len(paths), "upstream_commit": manifest["commit"],
            "protoc_version": version, "descriptor_sha256": descriptor,
            "external_imports": policy["external_imports"]}


def staged(protobuf_include):
    before = subprocess.check_output(["git", "write-tree"], cwd=ROOT)
    head = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT)
    local = subprocess.check_output(["git", "rev-parse", "--local-env-vars"],
                                    cwd=ROOT, text=True).splitlines()
    with tempfile.TemporaryDirectory(prefix="gz-schema-index-") as directory:
        export = Path(directory)
        subprocess.run(["git", "checkout-index", "--all", f"--prefix={export}/"],
                       cwd=ROOT, check=True)
        child_env = os.environ.copy()
        for name in local:
            child_env.pop(name, None)
        subprocess.run([sys.executable, "tools/test_validate.py"], cwd=export,
                       env=child_env, check=True)
        subprocess.run([sys.executable, "tools/validate.py", "--protobuf-include",
                        str(protobuf_include)], cwd=export, env=child_env, check=True)
    require(before == subprocess.check_output(["git", "write-tree"], cwd=ROOT)
            and head == subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT),
            "Git index or HEAD changed during staged validation")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--protobuf-include", type=Path, default=Path("/usr/include"))
    parser.add_argument("--staged", action="store_true")
    args = parser.parse_args()
    try:
        include = args.protobuf_include.resolve()
        if args.staged:
            staged(include)
        else:
            print(json.dumps(validate(ROOT, include), indent=2))
    except (ValueError, KeyError, OSError, subprocess.CalledProcessError) as error:
        parser.exit(1, f"Schema validation failed: {error}\n")


if __name__ == "__main__":
    main()
