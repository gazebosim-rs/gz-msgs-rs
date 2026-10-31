#!/usr/bin/env python3
# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0
"""Exercise schema drift, malformed imports, dependency identity and runtime refusal."""

import json
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest

from validate import ROOT, digest, validate


class SchemaValidation(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(prefix="gz-schema-test-")
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        shutil.copytree(ROOT / "schemas", self.root / "schemas")

    def check(self):
        return validate(self.root, Path("/usr/include"))

    def rehash(self, name):
        path = self.root / "schemas/manifest.json"
        manifest = json.loads(path.read_text())
        for entry in manifest["schemas"]:
            if entry["path"] == name:
                entry["sha256"] = digest(self.root / name)
        path.write_text(json.dumps(manifest))

    def test_exact_canonical_inventory_compiles(self):
        self.assertEqual(self.check()["schema_count"], 216)

    def test_changed_bytes_and_missing_schema_fail(self):
        path = self.root / "schemas/gz/msgs/double.proto"
        path.write_bytes(path.read_bytes() + b"\n")
        with self.assertRaisesRegex(ValueError, "schema hash mismatch"):
            self.check()
        path.unlink()
        with self.assertRaisesRegex(ValueError, "missing regular schema"):
            self.check()

    def test_unlisted_schema_and_symlink_fail(self):
        extra = self.root / "schemas/extra.proto"
        extra.write_text('syntax="proto3"; message Extra {}')
        with self.assertRaisesRegex(ValueError, "inventory"):
            self.check()
        extra.unlink()
        path = self.root / "schemas/gz/msgs/double.proto"
        original = path.read_bytes()
        path.unlink()
        extra.write_bytes(original)
        path.symlink_to(extra)
        with self.assertRaisesRegex(ValueError, "regular schema"):
            self.check()

    def test_rehashed_tag_drift_still_fails_descriptor_identity(self):
        name = "schemas/gz/msgs/double.proto"
        path = self.root / name
        path.write_text(path.read_text().replace("double data = 2;", "double data = 22;"))
        self.rehash(name)
        with self.assertRaisesRegex(ValueError, "descriptor identity"):
            self.check()

    def test_missing_import_fails_actual_compiler(self):
        name = "schemas/gz/msgs/double.proto"
        path = self.root / name
        path.write_text(path.read_text().replace("gz/msgs/header.proto", "gz/msgs/missing.proto"))
        self.rehash(name)
        with self.assertRaises(subprocess.CalledProcessError):
            self.check()

    def test_standard_any_dependency_identity_is_required(self):
        external = tempfile.TemporaryDirectory(prefix="gz-schema-external-test-")
        self.addCleanup(external.cleanup)
        external_root = Path(external.name)
        include = external_root / "google/protobuf"
        include.mkdir(parents=True)
        (include / "any.proto").write_bytes(Path("/usr/include/google/protobuf/any.proto").read_bytes() + b"\n")
        with self.assertRaisesRegex(ValueError, "external import hash"):
            validate(self.root, external_root)

    def test_runtime_package_is_refused(self):
        (self.root / "Cargo.toml").write_text("[package]\nname='runtime'\n")
        with self.assertRaisesRegex(ValueError, "Rust runtime"):
            self.check()


if __name__ == "__main__":
    unittest.main()
