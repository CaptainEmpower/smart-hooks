# 0001 — Consume prek rather than wrap it

**Status:** Accepted — 2026-09-15

## Context

smart-hooks currently presents itself as a front-end to [prek](https://github.com/j178/prek):
`smart-hooks run|install|list|validate` shell out to the `prek` binary, and the
README positions the project as "an enhanced layer over prek v0.2.20".

That position does not survive contact with the facts.

**prek has moved, and we have not.** Upstream is at v0.5.3 (2026-09-13), MIT, 8.4k
stars, in production at CPython, ruff, FastAPI, Airflow, Sentry, Godot and
home-assistant. We pin `0.2.20` in the README badge, in the `Cargo.toml` comment, in
the install instructions, and in `prek/detection.rs`, whose
`validate_prek_installation()` actively rejects any version that is not `0.2.x`.
Five minor releases of workspace mode, hook groups and priorities, native
`prek.toml`, built-in Rust hook implementations, `run --glob/--directory/--dry-run`,
and supply-chain safeguards on `prek update` have landed behind that pin.

**The wrapper is a strict, lossy subset.** It covers 4 of prek's ~14 subcommands.
One of the four, `smart-hooks validate`, invokes `prek validate` — a subcommand
prek does not have (it has `validate-config` and `validate-manifest`).

**The wrapper is unsound as a git hook.** `handle_prek_execution_failure` prints a
warning and returns `Ok(())`, so a failing hook run exits 0. The same pattern
appears in `execute_prek_install` and `execute_prek_validate`. Any repository
using `smart-hooks run` as its pre-commit hook has a hook that cannot fail.

There is no version of this wrapper that is worth maintaining. Every subcommand we
add is a subcommand we must keep in step with a project that ships faster than we
do, in exchange for no capability a user could not get by calling `prek` directly.

## Decision

Delete the delegation layer. smart-hooks becomes a hook that runs **under** prek,
not a front-end that runs prek.

- Remove `src/prek/` and the `run`, `install`, `list`, `validate` subcommands.
- Ship a `.pre-commit-hooks.yaml` describing only hooks that exist, so smart-hooks
  can be consumed from a `.pre-commit-config.yaml` like any other hook repository.
- Drop every version pin on prek. We no longer call it, so its version is not our
  concern.
- Document prek as the recommended runner, without claiming an integration.

## Consequences

**Good.** The exit-code defect disappears by construction — prek runs our binary
and reads its status, rather than us running prek and discarding its status. We
stop tracking a moving CLI surface. The install story shrinks to one binary.
Users get all of prek, current, instead of four subcommands of prek, frozen.

**Bad.** `smart-hooks run` is a breaking removal for anyone using it as their hook
entry point. The migration is to call `prek` directly, which is what the command
was doing anyway; the release notes must say so plainly.

**Accepted risk.** We give up the "one tool for everything" story. That story was
not true, and the projects that would have bought it are already running prek.

## Evidence

Exit-code loss, in a scratch repository with a hook whose entry is `bash -c 'exit 1'`:

```
prek run --all-files        → exit 1
smart-hooks run --all-files → exit 0
```

Missing subcommand, against the locally installed prek 0.2.20:

```
$ smart-hooks validate
✅ Validating configuration via prek...
warning: selector `validate` did not match any hooks
error: No hooks found after filtering with the given selectors
❌ Prek configuration validation failed      # and still exits 0
```

Upstream version and adoption: `gh api repos/j178/prek` and `gh api repos/j178/prek/releases`.
