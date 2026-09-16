# GitHub Workflows Documentation

This directory contains automated workflows for continuous integration, security analysis, and code quality assurance.

## Security & Analysis Workflows

### Daily Security Monitoring
| Workflow | Purpose | Schedule | Coverage |
|----------|---------|----------|----------|
| `semgrep-sast.yml` | SAST security analysis | Daily 2 AM UTC + PRs | OWASP Top 10, CWE Top 25, Rust security |
| `audit.yml` | Dependency vulnerability scanning | Daily midnight UTC + changes | CVE database, supply chain |

### Weekly Deep Analysis
| Workflow | Purpose | Schedule | Coverage |
|----------|---------|----------|----------|
| `weekly-security-scan.yml` | Git history secret scanning | Sundays 4 AM UTC | Historical commit analysis |

The unsafe-code analysis workflows this table used to list — `code-quality.yml`
(Rudra, Prusti), `miri-memory-safety.yml` and `kani-verification.yml` — were
inherited from the sibling repository `CaptainEmpower/git-mvh` and targeted its
crate, its modules and its proof harnesses. They could never run here. They are
removed rather than retargeted: this crate contains no `unsafe` and defines no
Kani proofs, and git-mvh still runs all three against its own code.

### Pull Request Workflows
| Workflow | Purpose | Trigger | Validation |
|----------|---------|---------|------------|
| `pr-checks.yml` | Core validation | Every PR | Tests, linting, compilation |
| `security.yml` | Secret detection | Every push/PR | Pre-commit secret scanning |
| `qodo-gate.yml` | Review findings closed | PR + review events | Blocks merge while a Qodo thread is unresolved |

`qodo-gate.yml` is deliberately separate from `pr-checks.yml`: it listens to
review and review-thread events as well as pushes, so resolving a Qodo thread
flips it green without a new commit or a full CI re-run. It also refuses to
report an all-clear on a PR no Qodo identity has reviewed — zero findings and
zero evidence of a review render as the same green check otherwise.

## Security Analysis Coverage

### OWASP Top 10 2021 & CWE Top 25
- **Tool**: Semgrep with comprehensive rulesets
- **Coverage**:
  - A01: Broken Access Control (CWE-22, CWE-352)
  - A02: Cryptographic Failures (CWE-327, CWE-328)
  - A03: Injection (CWE-79, CWE-89, CWE-78)
  - A04: Insecure Design (CWE-209, CWE-256)
  - A05: Security Misconfiguration (CWE-16, CWE-611)
  - A06: Vulnerable Components (CVE database)
  - A07: Authentication Failures (CWE-798, CWE-620)
  - A08: Software Integrity Failures (CWE-502, CWE-829)
  - A09: Logging Failures (CWE-532, CWE-778)
  - A10: Server-Side Request Forgery (CWE-918)

### Rust-Specific Security Patterns
- Memory safety violations
- Unsafe block analysis
- Use-after-free prevention
- Buffer overflow detection
- Integer overflow protection
- Panic safety verification

### Mathematical Verification
- Arithmetic operation correctness
- Division by zero prevention
- Overflow detection in statistical calculations
- Invariant preservation in data structures

## Workflow Dependencies & Sequencing

### Daily Schedule (UTC)
```
02:00 - Semgrep SAST analysis
00:00 - Trivy dependency scanning
```

### Weekly Schedule (Sundays UTC)
```
04:00 - Git history secret scanning (TruffleHog)
```

### PR Triggers
All security and analysis workflows run on pull requests to ensure code changes meet security standards before merging.

## Configuration

### Secrets Required
- `SEMGREP_APP_TOKEN` (optional): For enhanced Semgrep features
- `GITHUB_TOKEN`: Automatically provided for workflows

### Workflow Dispatch Options
Most workflows support manual triggering with customizable parameters:
- Verbosity levels
- Custom rule sets
- Scan depth configuration
- Output format options

## Security Integration

### GitHub Security Tab
**Note**: SARIF uploads to GitHub Security tab are currently disabled (requires GitHub Advanced Security subscription).
Security scan results are available as workflow artifacts and in step summaries for manual review.

### Pull Request Comments
Security workflows automatically comment on pull requests when findings are detected, providing immediate feedback to developers.

### Artifact Storage
All analysis results are stored as workflow artifacts with 30-day retention for historical analysis and compliance reporting.

## Maintenance

### Regular Tasks
- Review security findings weekly
- Update tool versions quarterly
- Adjust rules and thresholds based on false positive analysis
- Monitor workflow execution times and optimize as needed

### Tool Versions
All workflows pin tool versions for reproducibility. Update regularly while testing for breaking changes.
