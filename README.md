# smart-hooks

**Runs the Rust tests affected by your change, instead of all of them.**

A pre-commit hook for Cargo projects. Give it a list of changed files; it works
out which test modules can observe the change and runs only those.

It is a hook, not a hook runner. Use it under
[prek](https://github.com/j178/prek) (recommended) or
[pre-commit](https://pre-commit.com/).

## Install

```bash
cargo install --path smart-hooks
```

## Use

Add it to `.pre-commit-config.yaml`:

```yaml
repos:
  - repo: https://github.com/CaptainEmpower/smart-hooks
    rev: v0.3.0
    hooks:
      - id: smart-test-selector
```

Or run it directly:

```bash
smart-hooks test src/parser.rs src/lexer.rs
```

See the plan without running anything:

```bash
smart-hooks test --dry-run src/parser.rs
```

```
🔍 Analysing 1 changed file(s) for affected tests...
  unit: parser
```

`--json` emits the same plan for scripting:

```json
{
  "status": "planned",
  "files_analysed": 1,
  "test_plan": {
    "unit_tests": ["parser"],
    "integration_tests": false,
    "bdd_tests": false
  }
}
```

That is the whole CLI: `smart-hooks test [--dry-run] [--json] [FILES]...`.

## How selection works

Selection is **path-based**, derived from each changed file's module path:

- `src/parser.rs` → the `parser` unit-test filter
- `src/analysis/config.rs` → `analysis::config`
- `src/analysis/mod.rs` → `analysis`
- `src/main.rs`, `src/lib.rs` → no module of their own
- Files matching configured structural patterns (`/core/`, `/api/`, `/strategy/`, …)
  additionally select integration or scenario tests

Configure it with a TOML file matching `TestSelectorConfig` if the defaults do
not fit your layout.

## What it does not do yet

Stated plainly, because a test selector that quietly skips the wrong test is
worse than no selector:

- **It does not read your dependency graph.** Selection is path-based. If
  `src/parser.rs` changes and a test in `src/eval.rs` covers it, that test is not
  selected. Fixing this is the subject of
  [ADR 0004](docs/adr/0004-derive-the-impact-set-from-cargo-metadata.md): a
  `cargo metadata` reverse-dependency closure, with a full-run fallback wherever
  the graph cannot be trusted.
- **It has no soundness harness.** Nothing yet proves the selected set is a
  superset of the tests that would fail under a full run. Until it does, treat
  this as a fast pre-commit filter and keep a full `cargo test` in CI.
- **Rust and Cargo only.** Earlier versions advertised multi-language support;
  the analyzers existed but the CLI never reached them.

## Measured

Verified on a two-module demo crate where the unrelated module holds a
deliberately slow (1.5 s) test, warm target directory:

| | Wall clock |
|---|---|
| `cargo test` (full suite) | 1.55 s |
| `smart-hooks test src/calculator.rs` | 0.02 s |

And on this repository's own suite, warm, changing one file:

| | Wall clock |
|---|---|
| `cargo test --lib` (full, 107 tests) | ~10 s |
| `cargo test --lib analysis::config` after touching that file | 1.69 s |

Both numbers come from the commands shown; reproduce them before trusting them.
The saving scales with how slow your unrelated tests are and shrinks as
incremental compilation comes to dominate.

## Exit codes

`0` selection ran and every selected test passed · `1` a selected test failed ·
`2` usage error.

The hook fails your commit when a test fails. That sounds obvious; a previous
version printed the failure and exited 0.

## Design decisions

The scope, and what was removed to reach it, are recorded in
[docs/adr](docs/adr/):

| ADR | |
|---|---|
| [0001](docs/adr/0001-consume-prek-rather-than-wrap-it.md) | Consume prek rather than wrap it |
| [0002](docs/adr/0002-narrow-scope-to-rust-test-impact-selection.md) | Narrow scope to Rust test-impact selection |
| [0003](docs/adr/0003-delete-uncompiled-and-orphaned-code.md) | Delete uncompiled and orphaned code |
| [0004](docs/adr/0004-derive-the-impact-set-from-cargo-metadata.md) | Derive the impact set from `cargo metadata` |
| [0005](docs/adr/0005-strict-assertions-and-verifiable-claims.md) | Strict assertions and verifiable claims |

## Development

```bash
cargo test                                      # 111 tests
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --all -- --check
```

## License

MIT — see [LICENSE](LICENSE).
