# 0002 — Narrow scope to Rust test-impact selection

**Status:** Accepted — 2026-09-15
**Depends on:** [0001](0001-consume-prek-rather-than-wrap-it.md)

## Context

With the prek wrapper gone ([0001](0001-consume-prek-rather-than-wrap-it.md)), the
question is what is left that anyone would install.

The CLI currently advertises eleven subcommands and the hook manifest advertises
nine hooks. Most of them do not work: `smart-hooks lint`, `format`, `summary` and
`analyze dependencies` all exit 2 because the subcommands do not exist;
`bdd-feature-selector` invokes a `claude-bdd-selector` binary that is never built;
`smart-hooks check` returns success on a path that does not exist; `smart-hooks bdd`
lists `.feature` filenames and never reaches the Claude integration, whose
`select_bdd_features_with_claude` has no caller outside its own module.

Exactly one capability works end to end: map changed Rust files to `cargo test`
filters and run them.

### Is that capability differentiated?

**Against prek and pre-commit: yes.** Both run file-scoped hooks — "this file
changed, so run this linter on it". Neither answers "this file changed, so which
tests can observe the change". prek shows no sign of moving in that direction; it
is investing in runner speed, toolchain management and supply-chain safety.

**Against Nx `affected`: partly.** Nx computes impact at **project granularity** —
it uses git to find changed files, maps them to projects via the project graph,
then adds every project that depends on those. Touching one file in a library runs
that library's whole `test` target plus the test target of every dependent. It is
also inseparable from Nx: you adopt `nx.json`, per-project configuration and the Nx
task runner, and the real payoff is the computation cache (local or remote), where
unchanged targets are cache hits rather than re-runs. The project graph is built
from JS/TS imports and package manifests; Rust is only reachable through community
plugins. So Nx answers a coarser question, for a different ecosystem, at the price
of adopting a build system. A Rust team that already uses Cargo cannot reach for it.

The same argument applies to Turborepo and Bazel: workspace- or target-granular
impact, conditional on adopting the build system.

**Against Rust-native prior art: the niche is open but not empty.**
[`cargo-difftests`](https://github.com/dnbln/cargo-difftests) does selective
re-testing from per-test coverage profiles, which is strictly more principled than
anything here — it knows which lines each test actually executed. It is also
effectively dormant: 7 stars, last pushed 2024-05-19.

**The real bar is lower than any of these, and harder to clear.** Anyone can put
this in `.pre-commit-config.yaml` in five lines:

```yaml
- repo: local
  hooks:
    - id: tests
      name: tests
      entry: cargo test
      language: system
      pass_filenames: false
```

To justify existing, smart-hooks has to beat that. Today's implementation is a
filename-to-module-path string map whose defaults hard-code identifiers from an
unrelated project (`move_validator`, `history_processor`, `file_mover_service`,
`fast_export`) — see [0004](0004-derive-the-impact-set-from-cargo-metadata.md).
The measured saving is real (see Evidence), the mechanism is not yet trustworthy.

## Decision

smart-hooks is a **Rust test-impact selector**. One concern, stated honestly.

- Keep: take a list of changed files, produce a test plan, run it via Cargo, and
  report the plan as JSON when asked.
- Drop from the CLI and the hook manifest: `lint`, `format`, `summary`, `analyze`,
  `check`, `capabilities`, `schema`, `status`, `examples`, `bdd`.
- Drop the Claude-powered BDD feature selection and the multi-language analyzers.
  Both are aspirational surface area attached to a core that is not yet good
  enough to carry them.
- Publish exactly the hooks that the binary implements.
- Say "Rust and Cargo" everywhere the docs currently say "multi-language".

The multi-language and AI-assisted ideas are not refuted, only unfunded. If the
Rust core earns its keep, they can come back behind their own ADRs.

## Consequences

**Good.** The project becomes describable in one sentence and testable against one
benchmark: are we faster than `cargo test` on a real change, without missing a test
that would have failed? Everything we keep, we can defend.

**Bad.** This is a large breaking change and a large deletion. Anyone using the
removed subcommands loses them; since four of them never worked, the blast radius
is smaller than the diff suggests.

**Obligation.** Correctness now dominates speed. A selector that skips a test that
would have failed is worse than no selector, because it converts a red build into a
silent pass. [0004](0004-derive-the-impact-set-from-cargo-metadata.md) addresses
the mechanism; until it lands, the safe default on any uncertainty is to run more
tests, not fewer.

## Evidence

Measured on this repository, warm target directory, before any narrowing:

| Run | Wall clock |
|-----|-----------|
| `cargo test --lib` (full, 107 tests) | 10.13 s / 14.95 s |
| `cargo test --lib analysis::config` (7 tests, no source change) | 0.11 s / 0.07 s |
| `cargo test --lib analysis::config` after touching `analysis/config.rs` | 1.69 s |

So on this crate the incremental recompile costs ~1.6 s and the tests we skip cost
~10 s: filtering wins by roughly 6–9x, and the common objection that compilation
dominates Rust test time does not hold here.

Two caveats that keep this honest. The crate is small, and recompile cost grows
faster than test cost as a workspace grows. And part of that 10 s comes from tests
that shell out to `cargo` and `prek`, which this narrowing deletes — the baseline
must be re-measured on the narrowed suite and on a large external workspace before
any performance claim reaches the README.
