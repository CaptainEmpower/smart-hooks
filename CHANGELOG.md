# Changelog

All notable changes to smart-hooks will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
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