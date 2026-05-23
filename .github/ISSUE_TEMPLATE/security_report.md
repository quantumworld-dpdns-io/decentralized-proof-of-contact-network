---
name: Security Vulnerability Report
about: Report a security vulnerability in the proof-of-contact network
title: "[SECURITY] "
labels: security
assignees: quantumworld-dpdns-io

---

## ⚠️ IMPORTANT

<!--
    Please do NOT publicly disclose security vulnerabilities.
    Use this template to report security issues privately or through the
    appropriate private reporting channel.
-->

## Vulnerability Description

<!-- A clear and concise description of the security vulnerability. -->

## Affected Components

<!-- Which components are affected by this vulnerability? -->

- [ ] Cryptographic implementation (signatures, proofs, hashing)
- [ ] P2P networking (libp2p, peer discovery, DHT)
- [ ] REST API (authentication, authorization, input validation)
- [ ] Data storage (vector DB, DuckDB, Iceberg)
- [ ] AI integration (prompt injection, model security)
- [ ] Python bindings
- [ ] Dashboard (XSS, CSRF, authentication bypass)
- [ ] Docker/container configuration
- [ ] Dependencies (supply chain)
- [ ] Build/CI/CD pipeline
- [ ] Configuration (secrets, environment variables)
- [ ] Other (please specify)

## Severity Assessment

<!-- Estimate the severity of this vulnerability -->

- **CVSS Score** (if known): 
- **Impact**:
  - [ ] Critical (remote code execution, total network compromise)
  - [ ] High (unauthorized data access, privilege escalation)
  - [ ] Medium (limited information disclosure, denial of service)
  - [ ] Low (minor information leak, best practice violation)

## Attack Vector

<!-- Describe how an attacker could exploit this vulnerability. -->

## Proof of Concept

<!--
    If possible, provide a minimal proof of concept or reproduction steps.
    For critical vulnerabilities, please describe the issue without
    providing a full exploit.
-->

## Affected Versions

<!-- Which versions are affected? -->

## Suggested Fix

<!-- If you have a suggestion for how to fix the issue, please describe it here. -->

## Disclosure Timeline

<!-- When was this issue discovered? Have you disclosed it to anyone else? -->

## Contact Information

<!-- Optional: How can we reach you for follow-up questions? -->

## Acknowledgments

<!-- If you would like to be credited for finding this vulnerability,
     please let us know how you'd like to be acknowledged. -->
