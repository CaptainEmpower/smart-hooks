# 0005 — Strict assertions and verifiable claims

**Status:** Accepted — 2026-09-15

## Context

The README carries a badge reading *"tests 227 passing"*. `cargo test` exits 101
and runs zero tests ([0003](0003-delete-uncompiled-and-orphaned-code.md)). Nobody
lied; the badge is hand-written and nothing ever checked it.

The tests that do run are weaker than their count suggests. Eighteen assertions
cannot fail by construction:

- `assert!(true)` — 7 occurrences, e.g. `main.rs:197`,
  *"Compilation success indicates proper module separation"*.
- `assert!(x || !x)` — 4 occurrences, e.g. `detection.rs:101`,
  *"Always true, just testing it returns a bool"*.
- `assert!(result.is_ok() || result.is_err())` — 7 occurrences, e.g.
  `plan_executor.rs:139`.

Each of these is a test that passes whatever the code does. They inflate the count,
they turn green on regressions, and — for a tool whose entire purpose is deciding
which tests to skip — they are the exact failure mode we are asking users to trust
us not to have.

The same pattern runs through the prose. "227 tests passing", "AST + dependency
analysis", "no hard-coded business logic", "multi-language", "~5-15 seconds vs
~30-60 seconds" in `COMPARISON.md` — none produced by a command anyone can re-run.

## Decision

**Assertions must be able to fail.** Assert the value, not the shape: exact
outputs, exact error variants, exact exit codes. A test that only proves a function
returns is not a test; either assert something about what it returned or delete it.

- `assert!(true)`, `assert!(x || !x)`, and `assert!(r.is_ok() || r.is_err())` are
  removed, not adjusted. Where the behaviour is worth covering, replace with the
  real assertion; otherwise drop the test.
- Result-returning code is asserted with `unwrap()`/`expect()` on the value and a
  matched error variant on the failure path, never with an `is_ok() || is_err()`
  disjunction.
- Process-level behaviour is asserted on the exit code. The defect in
  [0001](0001-consume-prek-rather-than-wrap-it.md) — a hook that always exits 0 —
  was reachable only because no test looked at an exit status.

**Claims must name the command that produced them.** Any number in the README,
CHANGELOG or an ADR carries the invocation that yields it, or it does not ship.
Test counts come from CI, not from a hand-edited badge; where a static badge cannot
be generated, it is removed rather than guessed.

## Consequences

**Good.** The test count becomes a signal instead of decoration. Removing eighteen
unfailable assertions will lower the number and raise its meaning. Reviewers get a
rule they can apply mechanically instead of a judgement call.

**Bad.** Coverage drops on paths that were only nominally covered — most of the
`is_ok() || is_err()` cases sit on functions that shell out to `cargo` or `prek`,
where asserting real behaviour means either a fixture workspace or accepting the
gap. Some of those tests will be deleted with nothing put back, and the honest
consequence is a visible hole rather than a hidden one.

**Follow-on.** A lint that rejects these three patterns belongs in CI, so the rule
survives the reviewer who is in a hurry. Not in scope here; this ADR is the
statement of intent that such a lint would enforce.

## Evidence

```
$ grep -rn "assert!(true)" src tests | wc -l                      # 7
$ grep -rnE "assert!\(\w+ \|\| !\w+\)" src tests | wc -l          # 4
$ grep -rn "is_ok() || .*is_err()" src tests | wc -l              # 7
$ cargo test ; echo "EXIT=$?"                                     # EXIT=101, 0 tests run
```
