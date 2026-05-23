# Security Policy

## Supported Versions

| Version | Supported |
|---------|-----------|
| 0.1.x   | ✅ Active development |
| < 0.1   | ❌ Unreleased |

## Reporting a Vulnerability

We take the security of the proof-of-contact network seriously. If you discover a security vulnerability, please follow the responsible disclosure process below.

### How to Report

**Do not report security vulnerabilities through public GitHub issues.**

Instead, send a detailed report to:

- **Email**: security@poi.network (replace with actual security contact)
- **PGP Key**: [Download public key](https://poi.network/security-pgp.asc)
  - Fingerprint: `XXXX XXXX XXXX XXXX XXXX  XXXX XXXX XXXX XXXX XXXX`

### What to Include

- Type of vulnerability (e.g., buffer overflow, signature bypass, timing attack)
- Full steps to reproduce
- Proof of concept (if applicable)
- Impact assessment
- Suggested fix (if available)

### What to Expect

1. **Acknowledgment**: We will acknowledge receipt within 48 hours
2. **Triage**: We will triage the issue within 5 business days
3. **Fix**: We will develop and test a fix
4. **Release**: We will release a patched version
5. **Disclosure**: We will coordinate public disclosure timing

## Scope

### In Scope

- Core protocol: `poi-core` crate
- Node software: `poi-node` binary
- Networking layer: `poi-networking` crate
- API server: `poi-api` crate
- Python SDK: `poi-client` package
- MCP server: `poi-mcp-server` crate
- Build and deployment scripts

### Out of Scope

- Third-party dependencies (report to their maintainers)
- AI models (Ollama, Chroma, etc. — report to their projects)
- Infrastructure (host OS, network equipment, cloud providers)
- Physical security

## Cryptographic Keys

### Node Identity Keys

- Algorithm: Ed25519 (ed25519-dalek v2.1.1)
- Key size: 32 bytes secret, 32 bytes public
- Storage: PEM-encoded with `zeroize` on drop

### Post-Quantum Keys

- Algorithm: Dilithium5 (ML-DSA, via liboqs)
- Status: Feature-gated (`pqc` feature), not enabled by default
- Note: Signatures are ~2.5KB vs 64 bytes for Ed25519

## Bug Bounty

There is currently no bug bounty program. Security researchers who report valid vulnerabilities will be credited in release notes (with permission).

## Security Audits

Third-party security audits are planned before the 1.0 release. Current security is maintained through:
- Automated `cargo audit` and `cargo deny` in CI
- OWASP Top 10 Robot Framework test suites
- Code review requirements for all PRs
- `#[deny(unsafe_code)]` where possible (liboqs PQC requires FFI)

## Secure Development Lifecycle

1. **Design review**: Architecture review for security implications
2. **Implementation**: Follow coding standards (see `docs/development/coding-standards.md`)
3. **Testing**: Unit tests, property-based tests, integration tests
4. **Audit**: Automated dependency scanning, static analysis
5. **Release**: Signed releases, checksum verification

## Dependencies

Dependencies are managed with:
- `cargo audit` — Checks for known vulnerabilities
- `cargo deny` — License and advisory checks
- Dependabot — Automated dependency update PRs
- Trivy — Container image scanning
