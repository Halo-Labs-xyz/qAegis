# leanVM Integration Setup Guide

This guide explains how to enable the full leanVM integration with real Rust crates.

## Current Status

The leanVM integration is **fully enabled** with real Rust crates! All dependencies are active and the code compiles successfully.

## Available Crates

The following crates are available from the leanEthereum ecosystem:

1. **lean_vm** - Core virtual machine from `leanEthereum/leanMultisig`
2. **xmss** - XMSS signature aggregation from `leanEthereum/leanMultisig`
3. **p3-koala-bear** - KoalaBear field from `Plonky3` (lean-multisig branch)
4. **p3-field** - Field arithmetic from `Plonky3` (lean-multisig branch)
5. **lean-multisig** - Multisig aggregation from `leanEthereum/leanMultisig`

## Dependency Resolution

✅ **RESOLVED**: The `rand::distr` conflict has been fixed by:
1. Using `rand = "0.8"` (compatible with multilinear-toolkit)
2. Adding a Cargo patch to force multilinear-toolkit to use rand 0.8
3. Adding `rand_distr = "0.4"` for distribution support

## leanVM Integration Status

✅ **ENABLED**: All dependencies are active and working!

### Dependencies in Cargo.toml

```toml
# LeanVM dependencies (from leanEthereum/leanMultisig)
lean_vm = { git = "https://github.com/leanEthereum/leanMultisig.git", package = "lean_vm", optional = true }
xmss = { git = "https://github.com/leanEthereum/leanMultisig.git", package = "xmss", optional = true }
p3-koala-bear = { git = "https://github.com/TomWambsgans/Plonky3.git", branch = "lean-multisig", optional = true }
p3-field = { git = "https://github.com/TomWambsgans/Plonky3.git", branch = "lean-multisig", optional = true }
lean-multisig = { git = "https://github.com/leanEthereum/leanMultisig.git", optional = true }
rand_distr = "0.4"  # For distribution support

[features]
lean-vm = ["lean_vm", "xmss", "p3-koala-bear", "p3-field", "lean-multisig"]

# Patch multilinear-toolkit to use compatible rand version
[patch."https://github.com/leanEthereum/multilinear-toolkit.git"]
rand = { version = "0.8", features = ["std", "std_rng"] }
```

### Build with Feature Flag

```bash
# Build with leanVM support
cargo build --features lean-vm

# Or run with leanVM
cargo run --features lean-vm
```

### Current Implementation Notes

- ✅ Imports are enabled and working
- ✅ Poseidon16History is available (Poseidon24History not yet available in xmss crate)
- ✅ KoalaBear field arithmetic is available
- ✅ Bytecode execution structure is ready
- ⏳ Full bytecode parsing needs implementation (currently placeholder)

## Reference Implementation

See `docs/relrepos/ethlambda/` for a working implementation that uses:
- `lean-multisig` for signature aggregation
- `leansig` for signature operations
- Similar dependencies

## Alternative: Use ethlambda as Reference

The `ethlambda` directory shows how to properly integrate leanVM crates. You can:

1. Check `crates/common/crypto/Cargo.toml` for dependency versions
2. Use the same git revisions/commits for consistency
3. Follow the same pattern for initialization

## Testing

Once enabled, test the integration:

```bash
# Build with lean-vm feature
cargo build --features lean-vm

# Run tests
cargo test --features lean-vm

# Test API endpoints
curl -X POST http://localhost:5050/api/lean-vm/execute \
  -H "Content-Type: application/json" \
  -d '{"bytecode": "0x...", "public_inputs": [1, 2, 3]}'
```

## Next Steps

1. **Resolve dependency conflicts** - Fix `rand::distr` issue in multilinear-toolkit
2. **Enable crates** - Uncomment dependencies in Cargo.toml
3. **Update implementation** - Replace placeholders with real leanVM calls
4. **Test thoroughly** - Verify bytecode execution, Poseidon hashing, signature aggregation
5. **Integrate with QCP** - Connect quantum co-processing hooks

## References

- [leanVM Python Bindings](../docs/relrepos/leanVm-py/README.md)
- [ethlambda Implementation](../docs/relrepos/ethlambda/)
- [leanMultisig Repository](https://github.com/leanEthereum/leanMultisig)
- [Plonky3 Repository](https://github.com/TomWambsgans/Plonky3)

---

**Status**: ✅ Dependencies enabled and working!  
**Last Updated**: January 26, 2026  
**Build Status**: ✅ Compiles successfully with `--features lean-vm`
