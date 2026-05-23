# Testing Guide

## Test Architecture

The project uses multiple testing layers:

```
┌─────────────────────────────────────┐
│  Rust Unit Tests (#[cfg(test)])     │  <-- Fast, isolated
├─────────────────────────────────────┤
│  Rust Integration Tests (tests/)    │  <-- Cross-crate
├─────────────────────────────────────┤
│  Python Tests (pytest)             │  <── SDK tests
├─────────────────────────────────────┤
│  Robot Framework (acceptance)       │  <── API-level
├─────────────────────────────────────┤
│  Performance Benchmarks (criterion) │  <── Performance
└─────────────────────────────────────┘
```

## Running Tests

### Rust Unit Tests

```bash
# All workspace unit tests
cargo test --workspace

# Single crate
cargo test -p poi-core

# Single test
cargo test -p poi-core -- test_contact_proof_new

# With output
cargo test -- --nocapture

# With logging
RUST_LOG=debug cargo test
```

### Rust Integration Tests

Integration tests require a running node:

```bash
# Start node in background
cargo run -p poi-node -- --config config.dev.toml &

# Run integration tests (tagged with #[ignore])
cargo test --workspace -- --ignored

# Specific integration test
cargo test --workspace -- --ignored test_proof_lifecycle
```

### Doc Tests

```bash
cargo test --workspace --doc
```

### Python Tests

```bash
# Install test dependencies
pip install -e ".[dev,test]"

# Run all Python tests
pytest python-client/ tests/ -v

# With coverage
coverage run -m pytest
coverage report
```

### Robot Framework Tests

```bash
# Full acceptance test suite
make test-robot

# Specific test suite
make test-robot-all

# OWASP security tests
make test-robot-owasp

# With custom node URL
NODE_URL=http://localhost:3001 make test-robot
```

### Performance Benchmarks

```bash
# Run all benchmarks
cargo bench

# Specific benchmark
cargo bench --bench proof_verification

# With criterion HTML reports
# Open target/criterion/reports/index.html
```

### Code Coverage

```bash
# Generate coverage report
make coverage

# View HTML report
open target/tarpaulin/tarpaulin-report.html
```

## Testing Guidelines

### What to Test

- **Core types**: Serialization roundtrips, validation, edge cases
- **Cryptography**: Key generation, signing, verification, wrong keys
- **Orbital windows**: Creation, active/expired states, overlap detection
- **Proof chains**: Append, integrity verification, Merkle roots
- **Verification**: All four checks (signature, timestamp, window, chain)
- **API endpoints**: Status codes, error responses, pagination
- **SDK**: All client methods, error handling, async support

### What Not to Test

- Internal implementation details (test behavior, not methods)
- Third-party library functionality
- Configuration file parsing (test the config, not the file format)

## Writing Tests

### Rust Test Patterns

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // Arrange-Act-Assert pattern
    #[test]
    fn test_something() {
        // Arrange
        let input = setup_test_data();

        // Act
        let result = function_under_test(&input);

        // Assert
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), expected_value);
    }

    // Error case
    #[test]
    fn test_invalid_input() {
        let result = function_under_test(&invalid_data);
        assert!(matches!(result, Err(Error::InvalidInput(_))));
    }

    // Roundtrip
    #[test]
    fn test_serialization_roundtrip() {
        let original = create_test_object();
        let serialized = original.to_json();
        let deserialized = Object::from_json(&serialized).unwrap();
        assert_eq!(original, deserialized);
    }
}
```

### Integration Test Pattern

```rust
use poi_api_client::PoiClient;

#[tokio::test]
#[ignore]
async fn test_proof_lifecycle() {
    let client = PoiClient::new("http://localhost:3000");

    // Create window
    let window = client.create_window(...).await.unwrap();

    // Create proof
    let proof = client.create_proof("target", &window.id).await.unwrap();

    // Verify
    let report = client.verify_proof(&proof.id).await.unwrap();
    assert!(report.all_passed);
}
```

### Property-Based Testing

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_canonical_bytes_deterministic(
        proving in "[a-z]{10}",
        target in "[a-z]{10}",
    ) {
        let proof = make_proof(proving, target);
        let b1 = proof.canonical_bytes();
        let b2 = proof.canonical_bytes();
        assert_eq!(b1, b2);
    }
}
```

## CI Pipeline

Tests run automatically on GitHub Actions:

| Job | Command | Required |
|-----|---------|----------|
| Lint | `cargo clippy -- -D warnings` | Yes |
| Build | `cargo build --workspace` | Yes |
| Unit tests | `cargo test --workspace` | Yes |
| Coverage | `cargo tarpaulin` | No |
| Security audit | `cargo audit` | Yes |
| Docker build | `docker build` | Yes |
| Integration | `cargo test --ignored` | Scheduled only |

See `.github/workflows/ci.yml` for full pipeline definition.
