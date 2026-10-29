# Ordinary proto2 extension-range declarations

The formatter supports ordinary proto2 extension-range metadata without
changing descriptor loading or extension-field runtime behavior. It retains raw
range order and prints each range after fields and before reserved declarations,
using native inclusive output endpoints. A singleton is `extensions 1 to 1;`;
the maximum tag remains numeric. Raw exclusive endpoints and option presence are
not rewritten.

Reflection accepts several ranges that native protobuf rejects. Formatting checks
proto2 syntax, explicit positive increasing endpoints within the protobuf tag
limit, pairwise range overlap and intersections with reserved ranges. The
maintained reflection loader already rejects field-number intersections; those
rejections remain unchanged. Sorted copies and standard binary search validate
spans while emission uses the original order. This is a bounded formatting guard,
not a general descriptor validator.

A narrow prost-reflect accessor yields one dynamic options message per range in
original order. Absent options yield an empty message; an out-of-range iterator
request yields None. The existing maintained options decoder preserves unknown
fields and registered custom options. Unknown/custom options remain Unsupported;
unresolved uninterpreted options retain the loader's existing rejection. The
accessor does not change endpoint presence, source bytes, package identities or
Cargo dependencies. Its upstream archive, complete patch and authored-line
accounting are retained under the existing vendor policy and original licenses.

The pinned reference corpus has 123 complete native descriptor/CLI observations:
34 supported declarations, three native-admitted unknown-option omissions that
Rust explicitly rejects, and 86 native rejections. Eleven descriptors retain
existing Rust per-file loader errors (including unresolved options and field
conflicts); remaining native-invalid admitted ranges return an explicit formatter
error. Four focused Rust tests and Clippy pass. A clean downstream consumer still
uses standard prost traits and Any with no Cargo patch, preserving signed zero,
compatible decoding, unknown forwarding and integer defaults.

The exact existing legacy descriptor and historical CLI input now use complete
native-positive assertions after independent transition review. Descriptor bytes,
arguments and historical native observations are unchanged. Actual extension-field
declarations, groups and MessageSet remain unsupported. No upstream assertion
credit, canonical corpus, wire behavior, performance or MSG-02 acceptance claim
changes. Independent fresh-source/native/transition review passed; normal strict
staged and exact hosted qualification remain pending.
