# QVM Integration Summary

## Overview

The Quantum Virtual Machine (QVM) serves as the **overarching protocol stack** layer between the Aegis-TEE and the Sequencer, containing and enhancing the QRMS.

## Architecture Position

```
┌─────────────────────────────────────────────────────────────────┐
│                    QUANTUM AEGIS PROTOCOL STACK                 │
├─────────────────────────────────────────────────────────────────┤
│  ┌─────────────────────────────────────────────────────────┐    │
│  │               QVM ORACLE LAYER  ← YOU ARE HERE          │    │
│  │  ┌─────────────────┐ ┌─────────────────┐ ┌───────────┐  │    │
│  │  │ Quantum Circuit │ │ Noise Simulator │ │   Threat  │  │    │
│  │  │   Simulator     │ │ (Willow 105Q)   │ │   Oracle  │  │    │
│  │  └────────┬────────┘ └────────┬────────┘ └─────┬─────┘  │    │
│  │           └───────────────────┼────────────────┘        │    │
│  │                               ▼                         │    │
│  │              ┌─────────────────────────────┐            │    │
│  │              │   QRMS + APQC (contained)   │            │    │
│  │              └──────────────┬──────────────┘            │    │
│  └─────────────────────────────┼───────────────────────────┘    │
│                                ▼                                │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │              AEGIS-TEE LAYER  (below)                   │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                ▼                                │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │              BLOCKCHAIN LAYER (below)                   │    │
│  └─────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────┘
```

## Core Components

| Component | File | Purpose |
|-----------|------|---------|
| QVM Simulator | `qvm.rs:900+` | State vector quantum circuit simulation |
| Noise Model | `qvm.rs:183-229` | Hardware-accurate noise from calibration |
| Qubit Picker | `qvm.rs:231-870` | Optimal hardware qubit selection |
| Grover Oracle | `qvm.rs:1300+` | Symmetric crypto threat assessment |
| Shor Oracle | `qvm.rs:1360+` | Public key crypto threat assessment |
| Protocol Stack | `qvm.rs:1500+` | Full QVM-QRMS-TEE integration |

## Supported Quantum Processors

| Processor | Qubits | 2Q Error | T1 | ID |
|-----------|--------|----------|----|----|
| Willow Pink | 105 | 0.34% | 70μs | `willow_pink` |
| Weber | 72 | 0.60% | 25μs | `weber` |
| Rainbow | 53 | 0.90% | 20μs | `rainbow` |

## Key Features

### 1. Qubit Picking (NEW)
- Per-qubit error characterization from calibration
- Single-qubit Pauli, two-qubit Pauli, FSim, readout errors
- Multiple picking strategies (Balanced, MinimizeError, MaximizeCoherence)
- Automatic bad qubit/pair avoidance
- Circuit transformation to hardware qubits

### 2. Quantum Circuit Simulation
- State vector simulation up to ~25 qubits
- Noise model derived from real hardware calibration
- Support for common gates (X, Y, Z, H, CZ, CNOT, etc.)
- Pre-built circuits: Bell state, GHZ state, Grover search

### 3. Grover Threat Assessment
- Evaluates symmetric crypto (AES, SHA) vulnerability
- Quadratic speedup modeling: O(2^n) → O(2^(n/2))
- Physical qubit estimation with error correction
- Time-to-break calculation

### 4. Shor Threat Assessment
- Evaluates public key crypto (RSA, ECDSA, BLS)
- Logical qubit requirements (2n for RSA, 6n for ECDSA)
- T-gate count estimation
- Error correction overhead calculation

### 5. Protocol Stack Integration
- Automatic era transitions (PreQuantum → Nisq → FaultTolerant)
- Threat indicator generation for QRMS
- Bridge to Aegis-TEE Sequencer
- Recommended algorithm selection

## Threat Levels

| Level | Score | Description |
|-------|-------|-------------|
| None | 0 | No realistic threat |
| Theoretical | 1000 | Possible in theory only |
| Long-term | 3000 | >10 years away |
| Medium-term | 5000 | 5-10 years |
| Near-term | 7500 | 2-5 years |
| Imminent | 10000 | Current technology |

## Configuration

```toml
# qvm.toml
[processor]
type = "willow_pink"

[simulation]
default_repetitions = 3000
enable_circuits = true
apply_noise = true

[oracle]
assessment_interval_blocks = 100
auto_era_transition = true
```

## Quick Start

```rust
use qrms::qvm::{QvmProtocolStack, QvmConfig, QuantumProcessor};

// Create protocol stack
let config = QvmConfig {
    processor: QuantumProcessor::WillowPink,
    auto_era_transition: true,
    ..Default::default()
};
let mut stack = QvmProtocolStack::new(config);

// Run assessment
let risk = stack.assess_and_update();

// Bridge to TEE
stack.bridge_to_tee(&mut tee_sequencer);
```

## Files

| File | Description |
|------|-------------|
| `services/qrms/src/qvm.rs` | Core QVM implementation (~2000 lines) |
| `services/qrms/qvm.toml` | Configuration file |
| `docs/architecture/qvm_integration.md` | Full documentation |

## Qubit Picking Example

```rust
use qrms::qvm::{QubitPicker, QubitPickingStrategy, QuantumProcessor};

// Create picker with calibration data
let picker = QubitPicker::new(QuantumProcessor::Rainbow);

// Get best qubits for your circuit
let result = picker.pick_qubits(
    5,                                    // Need 5 qubits
    &[(0, 1), (1, 2), (2, 3)],            // Connectivity requirements
    QubitPickingStrategy::Balanced
);

println!("Selected: {:?}", result.selected_qubits);
println!("Estimated fidelity: {:.2}%", result.estimated_fidelity * 100.0);
println!("Avoid these qubits: {:?}", result.avoid_qubits);
```

## Integration Flow

```
1. QVM Oracle performs periodic assessment
   ↓
2. Grover/Shor threat analysis runs
   ↓
3. Threat indicators generated
   ↓
4. QRMS risk score updated
   ↓
5. Era transition if thresholds crossed
   ↓
6. Threat indicators bridged to Aegis-TEE
   ↓
7. TEE updates intelligence ordering
```

## References

- [Google Cirq QVM](https://quantumai.google/cirq/simulate/quantum_virtual_machine)
- [Willow Processor](https://blog.google/technology/research/google-willow-quantum-chip/)
- [NIST PQC](https://csrc.nist.gov/projects/post-quantum-cryptography)
