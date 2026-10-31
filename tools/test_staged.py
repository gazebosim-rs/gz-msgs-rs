#!/usr/bin/env python3
# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0
"""Verify that schema hooks check the intended index, not mutable worktree bytes."""

import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile

from validate import ROOT


def main():
    local = subprocess.check_output(["git", "rev-parse", "--local-env-vars"],
                                    cwd=ROOT, text=True).splitlines()
    env = {key: value for key, value in os.environ.items() if key not in local}
    env.update({"GIT_AUTHOR_NAME": "Schema test", "GIT_AUTHOR_EMAIL": "schema@example.invalid",
                "GIT_COMMITTER_NAME": "Schema test", "GIT_COMMITTER_EMAIL": "schema@example.invalid"})
    with tempfile.TemporaryDirectory(prefix="gz-schema-index-test-") as directory:
        root = Path(directory)
        for name in ["schemas", "tools"]:
            shutil.copytree(ROOT / name, root / name, ignore=shutil.ignore_patterns("__pycache__"))

        def git(*args, environment=env):
            return subprocess.run(["git", *args], cwd=root, env=environment,
                                  capture_output=True, text=True, check=True)

        git("init", "-q")
        git("add", ".")
        git("commit", "-qm", "Schema test fixture")
        command = [sys.executable, "tools/validate.py", "--staged"]

        def hook(environment, success):
            result = subprocess.run(command, cwd=root, env=environment, capture_output=True, text=True)
            if (result.returncode == 0) != success:
                raise AssertionError(result.stdout + result.stderr)

        schema = root / "schemas/gz/msgs/double.proto"
        schema.write_bytes(schema.read_bytes() + b"\n")
        context = dict(env, GIT_DIR=str(root / ".git"), GIT_WORK_TREE=str(root),
                       GIT_SSH_COMMAND="false")
        hook(context, True)  # An unstaged edit cannot alter the snapshot checked.
        alternate = dict(context, GIT_INDEX_FILE=str(root / "alternate-index"))
        git("read-tree", "HEAD", environment=alternate)
        git("add", "schemas/gz/msgs/double.proto", environment=alternate)
        hook(alternate, False)
        hook(context, True)  # The ordinary index was not replaced by the alternate.
        git("add", "schemas/gz/msgs/double.proto")
        hook(context, False)
    print("Staged worktree, explicit Git context and alternate-index controls passed")


if __name__ == "__main__":
    main()
