# Celo Contracts

Smart contracts and generated bindings for the Celo protocol.

## Generated bindings

Rust and TypeScript bindings are generated from the contract ABIs and checked
into this repository. The checked-in output is the source of truth for
downstream consumers, so it must stay in sync with the deployed interface.

### Reproducible generation

Binding generation must be deterministic on a clean checkout:

- Output is sorted by contract name, then by method name, then by argument
  order, so re-running generation never reorders entries.
- Generated files must not contain machine-local paths, absolute paths,
  timestamps, hostnames, or secrets. Only the contract name, ABI-derived
  signatures, and error discriminants are emitted.
- Regenerate with the pinned toolchain and commit the result. A clean checkout
  followed by regeneration must produce no diff.

### Compatibility checks

CI runs a binding compatibility check that compares freshly generated output
against the checked-in snapshot and fails when any of the following drift:

- public method names
- argument order
- return types
- error codes / error discriminants

A failure means the public interface changed without an explicit migration
note. To land an intentional change, add a version entry (see below) and
regenerate the bindings in the same commit.

### Compatibility exceptions

Any intentional breaking change to a public method signature or error
discriminant requires an explicit version entry. Add the entry to the
compatibility exceptions list, including the contract, the affected method or
error, the previous and new signature, and the version in which the change
ships. The compatibility check only passes a drift when a matching version
entry is present; undocumented drift always fails CI.

### Test plan

- Clean-checkout generation test: regenerate bindings from a clean checkout and
  assert the output matches the checked-in snapshot byte-for-byte.
- Intentional-drift fixture: a fixture that mutates a method name, argument
  order, return type, and error discriminant, asserting the compatibility check
  fails each case.
- Compile smoke tests: compile each generated binding target (Rust and
  TypeScript) to confirm the generated output builds.
