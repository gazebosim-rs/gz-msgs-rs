# Added production line coverage

The canonical policy requires strictly greater than 95% coverage of newly added
executable production lines independently of whole-file line and region coverage.
The earlier Messages runner enforced whole owned files and owned vendor patches,
but did not separately enforce added root-source lines. Historical green runs
must not be described as evidence of that separate denominator.

The staged runner compares the intended Git index with its original HEAD. It
builds a manifest containing every current `src/**/*.rs` and `build.rs` hash and
its added line numbers, then passes the manifest outside the exported source to
the verifier. Initial commits compare against empty source. The parent preserves
its index and Git context; the child still removes only repository-local Git
environment settings before foreign-repository checks. Neither unstaged fixes
nor a different ordinary index can choose the changed lines.

Hosted CI fetches the required history and supplies the push's `before` commit
or the pull request's base commit. A new branch's all-zero base measures all
production lines. Missing named bases fail. An ordinary direct verifier call
uses `HEAD^` unless an explicit `--base` is supplied; an exported source without
Git requires the staged runner's `--changed-lines` manifest.

The authoritative manifest remains in a private temporary directory throughout
build/test execution; only the post-validation evidence copy is written under
`target/`. Before checks, the verifier validates the complete production inventory and each
source hash. After the existing whole-file and vendor gates, the changed-line
check intersects the added lines with LLVM's executable line records and applies
`covered * 100 > total * 95` separately for each changed file. Missing or empty
source instrumentation fails. Compiler-unmapped added syntax is listed in the
proof. A file without executable additions is explicitly `not_applicable`, with
zero counts; it is never reported as 100% coverage. The whole-file gate remains
mandatory even for metadata-only changes.

Each staged run retains coverage JSON, LCOV, owned vendor results, the changed
production proof and its source manifest under ignored `artifacts/staged/` in a
unique directory bound to the staged tree. A result records checks, evidence and
overall status separately. A successful child with missing required evidence
fails retention; a failed child remains failed even if some files were produced.
Early export/manifest failures also retain failed/partial results. Original
failures are preserved if retention also fails. Hosted CI uploads the
same coverage files alongside its existing source-package artifacts.

Focused controls exercise exactly-95 and below-95 failures, 96 passing, a 99%
whole-file result hiding a missed newly added line, missing source/instrumentation,
stale hashes, malformed line inventories and real Git bases/new files. A real
tracked target-input collision demonstrates that overwriting the generated copy
cannot mask a newly missed line; deliberate authoritative-input and source
mutations also fail after an otherwise successful child. Existing
real worktree, alternate index, foreign Git, export-byte/mode and partial-staging
regressions remain mandatory. This tooling change does not add production
exclusions, change numerical thresholds, or qualify any new message behavior.


Independent [source review](reviews/changed-root-coverage.json) verified the
isolated corrected policy export, seven changed-line controls, ten snapshot
controls and the unchanged legacy gates. The first unapproved draft failed to
retain early export/manifest failures and placed the authoritative mask in a
build output path; the corrected source closes both issues with real failure
and collision controls. It changes no Rust source or existing test assertions.
Its combination with the reviewed Header/stamp implementation passed the exact
full source gate: 131 Rust tests including 34 CLI, 6,598 native outcomes, strict
owned whole-file/vendor coverage and 63/63 added executable production lines.
The retained staged result reports passed checks and complete evidence. Final
metadata binding, normal protected hook and exact hosted qualification remain
pending.
