# Changelog

All notable changes to smart-hooks will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed — narrowed to Rust test-impact selection (breaking)

smart-hooks is now a single-purpose pre-commit hook that runs the Rust tests
affected by a change. It is consumed *under* a hook runner such as prek rather
than acting as a front-end to one. Rationale in [docs/adr](docs/adr/).

- **Removed the prek delegation wrapper** and the `run`, `install`, `list` and
  `validate` subcommands. The wrapper covered 4 of prek's ~14 subcommands, pinned
  prek `0.2.20` (upstream is `v0.5.3`), invoked a `prek validate` subcommand that
  does not exist, and **discarded prek's exit code** — a failing hook exited 0.
  Call `prek` directly instead. (ADR 0001)
- **Removed the rest of the CLI**: `lint`, `format`, `summary`, `analyze`,
  `check`, `capabilities`, `schema`, `status`, `examples`, `bdd`. Four of these
  never existed as subcommands and exited 2 when the published hooks invoked
  them. The CLI is now `smart-hooks test [--dry-run] [--json] [FILES]...`. (ADR 0002)
- **Removed Claude-assisted BDD selection and the multi-language analyzers.**
  The BDD path was never wired to the CLI and `analyze` failed on a Cargo virtual
  workspace root. (ADR 0002)
- **`.pre-commit-hooks.yaml` now publishes one hook**, `smart-test-selector`,
  down from nine. Of the previous nine, four named subcommands that did not
  exist, one named a binary that was never built, and one passed a literal
  `selective` that was silently read as a filename.

### Fixed

- **Module extraction ignored repository-relative paths.** `extract_module_name`
  required a literal `/src/`, so `src/parser.rs` — the path shape prek and
  pre-commit actually pass — matched nothing and the hook selected no tests at
  all. Path matching is now segment-aware. The same applied to
  `is_core_functionality_file`.
- **`bdd_structural_patterns` was computed and never read** by the test planner,
  so the "domain-agnostic, configurable" selection path had no effect.
  `should_run_bdd_tests` now consults it.
- **`cargo test` did not compile**, exiting 101 without running a single test:
  three `hotreload` test targets imported a module that had been commented out of
  `lib.rs`. (ADR 0003)
- **Two unit tests invoked the `claude` CLI** and asserted bounds on the model's
  reply, so they passed only where the binary was absent. Both now exercise the
  deterministic static path.

### Removed

- `src/hotreload/` (7,923 lines), excluded from the library since `4878c29` and
  never reachable; recoverable from git history if revived behind a new ADR.
- 2,475 lines of orphaned pre-refactor modules that no `mod` declaration
  referenced.
- Documentation describing removed functionality: `HOTRELOAD_ARCHITECTURE.md`,
  `PREK_INTEGRATION.md`, `COMPARISON.md`, `DESIGN.md`, `DATASHEET.md`,
  `PERFORMANCE.md`, `BDD_DETECTION_COMPARISON.md`,
  `CLAUDE_BDD_FEATURE_SELECTION.md`, `STRICT_BDD_VALIDATION.md`.

### Previously (superseded by the narrowing above)

### Added
- **SRP Module Refactoring**: Comprehensive codebase refactoring following Single Responsibility Principle
  - Transformed monolithic `main.rs` (1,239 LOC) into 11 focused modules
  - Added **78 new inline unit tests** across all refactored modules
  - Created `prek/` module with 4 sub-modules for prek integration (815 LOC total)
  - Created `smart_analysis/` module with 4 sub-modules for intelligent analysis (725 LOC total) 
  - Created `examples/` module with 3 sub-modules for usage examples (767 LOC total)
  - All modules comply with ≤365 LOC guideline for maintainability
  - Achieved **227 total tests** (121 binary + 106 library) with 99.5% success rate
- **Prek Integration v0.2.20**: Seamless integration with [prek](https://github.com/j178/prek) pre-commit framework
  - Zero-coupling CLI delegation approach
  - Auto-detection of prek installation
  - Intelligent fallback to smart analysis when prek unavailable
  - Enhanced pre-commit commands: `run`, `install`, `list`, `validate`
  - Complete documentation in [PREK_INTEGRATION.md](PREK_INTEGRATION.md)
- Smart git hook generation when prek not available
- Graceful degradation with clear installation guidance
- Support for standard `.pre-commit-config.yaml` configuration
- Hybrid workflows combining prek infrastructure with smart analysis

### Enhanced
- CLI interface now includes enterprise-grade pre-commit commands
- Installation options for both standalone and integrated usage
- Configuration validation for both prek and smart-hooks configs
- Error handling with automatic fallback strategies

### Technical
- CLI delegation architecture using external process execution
- Stable prek@0.2.20 integration (not unstable master)
- Zero local dependencies or workspace coupling
- Production-ready enterprise integration patterns

## [0.2.0] - Previous Release

### Added
- Intelligent test selection based on code changes
- BDD feature selection with Claude AI integration
- Multi-language dependency analysis
- Hot-reload optimization capabilities
- Configurable pattern matching
- Domain-agnostic architecture

### Features
- Smart test selector for Rust projects
- Cucumber/BDD integration with semantic tag mapping
- Claude AI powered code analysis (optional)
- Multi-language support (TypeScript, Python, PHP, Rust)
- Zero-configuration setup with sensible defaults
- Comprehensive test coverage (44 passing tests)

---

## Integration Architecture Summary

The prek integration represents a major architectural enhancement that provides:

1. **Enterprise Compatibility**: Full compatibility with existing pre-commit workflows
2. **Zero Coupling**: No local dependencies, perfect standalone operation  
3. **Intelligent Enhancement**: Smart analysis capabilities augment standard hook execution
4. **Graceful Degradation**: Works perfectly with or without prek installed
5. **Production Ready**: Battle-tested integration patterns for enterprise use

This makes smart-hooks the perfect solution for teams wanting to:
- Enhance existing pre-commit setups with intelligence
- Migrate from pre-commit to a faster, smarter solution
- Get enterprise-grade hook infrastructure with AI-powered analysis
- Maintain compatibility while gaining advanced capabilities

**Built with ❤️ for the future of intelligent development workflows**