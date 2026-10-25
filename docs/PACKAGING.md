# Source distribution contract

The supported source distribution preserves the Cargo manifest's direct vendored
paths. `publish = false` stays enabled. Cargo's registry-oriented `.crate` format
rewrites those paths to unpatched registry versions and excludes both nested
vendor packages. An extracted ordinary `.crate` reproduced two signed-zero/default
predicate test failures. It must not be published as a compatible package.
Crates.io publication needs a separately reviewed upstream/fork package identity
and release policy; no override table may be required from downstream consumers.

Git/path consumers and the immutable source archive retain the reviewed
prost-derive/prost-reflect fixes without consumer Cargo patches. All original
vendor licenses/notices, patch provenance, canonical schemas, Cargo.lock, source,
tests and toolchain declarations are included unchanged. Root-owned files remain
Apache-2.0 with Open Source Robotics Foundation attribution.

## Produce and verify

Run from a Git checkout with Python3.12+, Git and the qualified Rust1.94.1/protoc
3.21.12 build tools. The archive uses only the named commit's blobs, not checkout
transforms, export-ignore attributes, unstaged edits or untracked files.

```sh
revision="$(git rev-parse HEAD)"
python3 scripts/source-package.py --revision "$revision"
(cd artifacts/distribution && sha256sum -c "gz-msgs-$revision.tar.gz.sha256")
python3 scripts/test-source-package.py \
  --archive "artifacts/distribution/gz-msgs-$revision.tar.gz" \
  --revision "$revision" --fresh-cache
```

The gzip/tar timestamps, owner fields and ordering are deterministic. A regression
produces byte-identical archives in different output directories under the same
Python/zlib environment; cross-version compression and compiled-binary
reproducibility are not claimed. The sidecar SHA256 identifies the archive bytes.

SOURCE_MANIFEST.json records exact Git modes/blob IDs/SHA256s, the tree and the
original commit object. Verification checks the trusted expected commit, every
member's path/type/mode/content and its Git object hash; Git reconstructs the tree
and must match the original commit. Noncanonical paths are rejected without collapsing dot/empty components; the
reconstructed index must account for every mode/OID/stage/path, including names
Git would silently ignore. Missing/extra/changed inputs fail. Extraction
uses Python's data filter and rejects links outside the revision directory.
Regression cases retain CRLF bytes/export-ignore inputs/executable modes/symlinks,
exclude unstaged/untracked files, reject wrong revisions, changed blobs/modes,
added or omitted files and escaping symlinks.

The install proof uses a fresh extracted source directory, optional empty private
Cargo home and a local install prefix. Fresh-cache mode fetches only locked public
dependencies, then installs offline. It runs all nine installed CLI native-output
observations and a clean downstream consumer proving signed zero, standard Any
traits and nested unknown forwarding without patches. The executable's actual
Linux dynamic-library closure must contain no Gazebo or protobuf C++ libraries.
No shared toolchain, Cargo configuration or user install prefix is changed.
Proof records the commit, tree, archive SHA, tool versions and runtime libraries.

CI runs archive regressions inside the strict source gate, then creates the exact
CI commit archive and performs the fresh-cache install/consumer proof. Only
successful source archives, sidecars and proof JSON are uploaded as revision/run
attempt artifacts. These are source/CI artifacts, not an enabled crates.io release
or a completed clean-machine/platform/binary-package release matrix.
