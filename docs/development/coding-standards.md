# Coding Standards

## Language: Rust

The project is primarily written in Rust. Python is used for scripting, SDK, and tests.

## Rust Standards

### Naming Conventions

| Item | Convention | Example |
|------|------------|---------|
| Types | PascalCase | `ContactProof`, `OrbitalWindow` |
| Functions/Methods | snake_case | `verify_signature`, `canonical_bytes` |
| Variables | snake_case | `proof_id`, `public_key` |
| Constants | SCREAMING_SNAKE_CASE | `MAX_TIME_DRIFT_SECS` |
| Traits | PascalCase | `ContactProofExt`, `KeyPairExt` |
| Modules | snake_case | `orbital_window`, `contact_proof` |
| Errors | PascalCase | `Error::InvalidSignature` |
| Type parameters | single uppercase | `T`, `E` |

### Code Organization

```
src/
  lib.rs          # Public API, re-exports
  types.rs        # Core data types
  module.rs       # Implementation module
```

- Each crate has a `lib.rs` that re-exports public API
- Modules are organized by concern, not by type
- Public items are documented with doc comments (`///`)

### Error Handling

```rust
// Use thiserror for error enums
#[derive(Error, Debug)]
pub enum Error {
    #[error("Invalid proof: {0}")]
    InvalidProof(String),
    #[error("Window expired")]
    WindowExpired,
}

pub type Result<T> = std::result::Result<T, Error>;

// Return specific errors
fn verify(proof: &Proof) -> Result<bool> {
    if condition {
        return Err(Error::InvalidProof("reason".into()));
    }
    Ok(true)
}
```

### Documentation

```rust
/// Verifies the Ed25519 signature on a contact proof.
///
/// Decodes the base64-encoded public key and signature, recomputes
/// canonical bytes, and verifies the signature.
///
/// # Errors
/// - `Error::InvalidKey` if the public key is not valid base64 or not 32 bytes
/// - `Error::InvalidSignature` if the signature is malformed
pub fn verify_signature(proof: &ContactProof, public_key: &PublicKey) -> Result<bool>
```

All public API items must have doc comments. Include `# Errors` sections for functions returning `Result`.

### Testing

- Tests are in `#[cfg(test)] mod tests` blocks at the bottom of implementation files
- Every public function must have at least one test
- Use descriptive test names: `test_[function]_[scenario]`
- Prefer `assert!` / `assert_eq!` over `unwrap()` in test assertions

### Imports

```rust
// Standard library first
use std::collections::HashMap;

// External crates (alphabetical)
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// Internal crate modules
use crate::types::{ContactProof, NodeId};
```

## Python Standards

### Conventions

- Python 3.11+ with type hints
- Follow PEP 8, enforced by Ruff
- Use Pydantic for data models
- Use httpx for HTTP requests

### Type Hints

```python
from __future__ import annotations
from typing import Any
from datetime import datetime

def create_proof(self, target_node: str, window_id: str, purpose: str = "") -> Proof: ...
```

## General Standards

### Formatting

- Rust: `cargo fmt` (rustfmt)
- Python: `ruff format`
- TOML: 2-space indentation
- Markdown: 80-character line wrap for readability

### Linting

```bash
# Rust
cargo clippy --workspace -- -D warnings

# Python
ruff check .
ruff format --check .
```

### Commit Messages

```
<type>(<scope>): <description>

<optional body>
```

Types: `feat`, `fix`, `docs`, `test`, `refactor`, `perf`, `ci`, `chore`

Examples:
```
feat(core): add PQC signature verification
fix(api): handle empty proof list in search endpoint
docs: add API reference for proof endpoints
```

### Feature Flags

New functionality should be behind feature flags when:
- Adding optional dependencies
- Experimental/not-yet-stable features (e.g., PQC)
- Platform-specific code

```toml
[features]
default = []
pqc = ["dep:liboqs"]
```

```rust
#[cfg(feature = "pqc")]
pub fn pqc_sign(&self, data: &[u8]) -> Result<PqcSignature>;
```

### Security

- Zeroize secret keys on drop (`#[zeroize(drop)]`)
- Never log or debug-format secret key material
- Use constant-time comparison for signature verification
- Validate all user inputs at API boundaries
- Keep dependencies updated with `cargo audit`
