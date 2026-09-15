# Architecture Decision Records

Each ADR records one decision, the context that forced it, and what it costs us.
They are immutable once accepted: to change a decision, add a new ADR that
supersedes the old one rather than editing history.

| ADR | Title | Status |
|-----|-------|--------|
| [0001](0001-consume-prek-rather-than-wrap-it.md) | Consume prek rather than wrap it | Accepted |
| [0002](0002-narrow-scope-to-rust-test-impact-selection.md) | Narrow scope to Rust test-impact selection | Accepted |
| [0003](0003-delete-uncompiled-and-orphaned-code.md) | Delete uncompiled and orphaned code | Accepted |
| [0004](0004-derive-the-impact-set-from-cargo-metadata.md) | Derive the impact set from `cargo metadata` | Accepted |
| [0005](0005-strict-assertions-and-verifiable-claims.md) | Strict assertions and verifiable claims | Accepted |

## Format

Short MADR: **Status**, **Context**, **Decision**, **Consequences**, and where a
decision rests on measurement, an **Evidence** section naming the command that
produced the numbers. Claims without a reproducible command do not belong in an
ADR.
