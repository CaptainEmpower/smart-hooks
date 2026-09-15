# 0004 — Derive the impact set from `cargo metadata`

**Status:** Accepted — 2026-09-15
**Depends on:** [0002](0002-narrow-scope-to-rust-test-impact-selection.md)

## Context

[0002](0002-narrow-scope-to-rust-test-impact-selection.md) commits us to one
capability. This ADR decides how it works, because the current mechanism cannot be
the answer.

Selection today is substring matching on file paths. `TestSelectorConfig::default()`
ships patterns lifted from an unrelated project — `move_validator.rs`,
`history_processor.rs`, `file_mover_service.rs`, `/fast_export/` — so on any other
repository the defaults match nothing and the tool silently selects nothing. The
README calls this "AST + dependency analysis"; the parser is
`line.starts_with("use ")` over raw text, which mis-reads `use` inside strings,
comments, `cfg`-gated blocks and macro bodies alike. And `smart-hooks analyze`
fails outright on a Cargo virtual workspace root with *"No package metadata found"* —
the standard layout for exactly the monorepos that would most benefit.

The requirement that matters is **soundness**, not speed. A selector that skips a
test that would have failed turns a red build green, which is worse than having no
selector: the user has traded a slow signal for a fast lie. Speed is only worth
having on top of a selection we can argue is complete.

## Decision

Build the impact set from Cargo's own view of the workspace, in two levels, with an
explicit bias toward running too much.

**Level 1 — crates (default).** Run `cargo metadata` to get workspace members,
their `manifest_path`, each target's `src_path`, and the inter-member dependency
edges. Map every changed file to its owning package by longest-prefix match on the
package root, then take the reverse-dependency closure over workspace members. The
result is the `-p` set for `cargo test`. This is the same question Nx `affected`
answers, resolved against Cargo instead of a bespoke graph, and it is correct by
construction: if a crate cannot observe the change through the dependency graph, it
cannot fail because of it.

**Level 2 — modules within a changed crate (opt-in).** Inside the crates selected
at level 1, resolve the changed files to module paths using the real module tree —
parsed with `syn`, honouring `#[path]`, `mod` declarations and `cfg` attributes,
not text matching — and narrow `cargo test` with those filters. Opt-in via
`--granularity module`, because narrowing within a crate is where we are most
likely to be wrong.

**Fall back to the full workspace** whenever the graph cannot be trusted. Any
change to `Cargo.toml`, `Cargo.lock`, `build.rs`, `rust-toolchain.toml`,
`.cargo/config.toml`, or any file we cannot attribute to a package; any crate that
is or depends on a proc-macro crate; any use of `include!`, `include_str!`,
`include_bytes!` or `env!`, which create dependencies the module graph does not
see. Detecting these is cheap. Guessing about them is not.

**Report the plan before running it.** `--dry-run` prints the selected packages,
filters, and the reason for each — including "fell back to full run because X". A
selector whose reasoning is invisible cannot be debugged when it is wrong.

## Alternatives considered

**Coverage-based selection**, as [`cargo-difftests`](https://github.com/dnbln/cargo-difftests)
does it: instrument the test run, record which lines each test executed, re-run only
tests whose covered lines changed. Strictly more precise than a dependency graph —
it prunes within a crate without guessing. Rejected for now because it needs an
instrumented baseline run and a profile store that must be kept fresh, which is a
much larger commitment than we can justify before the crate-level version has
proven useful. Worth revisiting in its own ADR.

**Keeping the text parser, fixed.** Rejected: the failure mode is silent
under-selection, and there is no version of substring matching we could argue is
complete.

**Calling `nx affected` or Bazel.** Rejected: both require adopting the build
system, which is the cost [0002](0002-narrow-scope-to-rust-test-impact-selection.md)
exists to avoid.

## Consequences

**Good.** Selection works on any Cargo workspace with no configuration, because the
configuration is the workspace. Virtual manifests work, since that is what
`cargo metadata` is for. The "dependency analysis" claim becomes true.

**Bad.** `cargo metadata` costs real time on a cold cache, which eats into the
saving on small workspaces — the dry-run output must report it so we can see when
we are losing. Taking on `syn` and `cargo_metadata` adds dependencies to a crate
that currently has almost none.

**Obligation.** Soundness needs a harness, not an assertion. Before this is
recommended for anyone's pre-commit hook, we need a differential test over a sample
of real commits checking that the selected set is a superset of the tests that
actually fail under a full run. A selector without that harness is a guess with a
progress bar.
