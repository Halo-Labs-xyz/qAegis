# Contributing to QuantumAegis

## Development Setup

1. Clone the repository
2. Install dependencies (Rust, Foundry, Docker)
3. Follow deployment guide in `docs/deployment/`

## Code Style

- Rust: Follow `rustfmt` defaults
- Solidity: Follow Solidity style guide
- Commits: Use conventional commits

## Testing

```bash
# Rust tests
cd services/qrms && cargo test

# Contract tests
cd contracts && forge test
```

## Pull Requests

- Include tests for new features
- Update documentation
- Follow existing code patterns
