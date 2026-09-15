# 0003 — Delete uncompiled and orphaned code

**Status:** Accepted — 2026-09-15

## Context

41% of `src/` is not compiled into the library or the binary.

**The hot-reload subsystem — 7,923 lines** across `src/hotreload/` — is commented
out of `src/lib.rs` (`// Temporarily disabled due to missing dependencies (blake3,
notify, tracing)`), disabled by commit `4878c29` during the SRP refactoring. It has
never been reachable since. Three integration-test targets still import it
(`hotreload_bdd`, `hotreload_integration`, `hotreload_performance`, 1,838 lines),
so they fail to compile with `unresolved import smart_hooks::hotreload`. Because
one target failing to build aborts the whole run, **`cargo test` exits 101 and runs
zero tests**, while CI's `cargo test --test "*" --release` gate has been red for the
same reason. `HOTRELOAD_ARCHITECTURE.md` is 28 KB describing this code as if it
shipped.

**Orphaned root modules — 2,475 lines** are declared by neither `lib.rs` nor
`main.rs`, so `rustc` never sees them: `main_original.rs` (1,239),
`conditional_compilation_checker.rs` (332), `claude_bdd_selector.rs` (230),
`lib_old.rs` (177), `smart_test_selector.rs` (181),
`smart_test_selector_refactored.rs` (49), `selective_unit_tests.rs` (97),
`integration_tests_if_needed.rs` (170). They are pre-refactor copies left behind as
an informal undo buffer. `smart_test_selector.rs` is where the `crates/git-mvh/src/`
hard-coding lives, and `lib.rs` still opens with `/// Git-MVH Pre-commit Hooks
Library`.

The earlier commits called both situations temporary (`e1ad925`, "test: temporarily
disable failing integration tests during SRP refactoring"). Nine months of
temporary is a decision that was never written down.

Uncompiled code is worse than absent code. It reads as if it works, it is cited in
the README, it is counted in the "227 tests passing" badge, and no compiler or test
run contradicts any of it.

## Decision

Delete it, rather than repair it.

- Delete `src/hotreload/` and its three test targets.
- Delete `HOTRELOAD_ARCHITECTURE.md`.
- Delete the eight orphaned root modules.
- Fix the one genuinely failing library test rather than deleting it, so that the
  post-deletion suite is green on its own merits.

Deletion is reversible: `git log` keeps every line, and any ADR that resurrects
hot-reload caching can recover the code from history. What is not reversible is the
cost of carrying it — it is the largest single reason the test suite does not run.

## Consequences

**Good.** `cargo test` compiles and runs. CI's integration gate can go green. The
source tree stops advertising capabilities the binary does not have. Future
refactors stop dragging three copies of each module.

**Bad.** Real design work in the hot-reload tree goes cold. Content-hash cache keys
and the disk LRU are the most interesting code in the repository, and hook-result
caching is genuinely absent from prek — a plausible future differentiator. Losing
it from the working tree raises the cost of picking it up again.

**Mitigation.** This ADR is the pointer: the implementation is recoverable at
commit `4878c29`, and reviving it requires a new ADR that states which dependencies
(`blake3`, `notify`, `tracing`) are being taken on and why the caching is correct.

## Evidence

```
$ cargo test ; echo "EXIT=$?"
error[E0432]: unresolved import `smart_hooks::hotreload`   (×3)
EXIT=101

$ cargo test --lib --bins
test result: FAILED. 106 passed; 1 failed; 0 ignored
  analysis::claude_bdd_detector::tests::test_hybrid_analysis
```

Line counts: `find src -name '*.rs' | xargs wc -l` (25,620 total),
`find src/hotreload -name '*.rs' | xargs wc -l` (7,923),
`wc -l tests/hotreload_*.rs` (1,838).
