# QRMS Service

Quantum Resistance Model System - Core monitoring and PQC service.

## Features

- Real PQC signatures (ML-DSA-87, SLH-DSA-256s)
- Hybrid ECDSA + PQC dual signatures
- 12-category threat monitoring
- Adaptive algorithm rotation
- Web GUI and CLI interfaces
- Hybrid quantum-classical execution mode (`hybrid_quantum`)
- Zero-retention persistence controls with consent model
- Governance adjudication with append-only transparency hash chain
- Evidence-gated claims manifest generation

## Usage

```bash
# Run server
cargo run --release

# Run CLI monitor
cargo run --release --bin qrms-cli

# Run hybrid solve CLI
cargo run --release --bin hybrid-cli -- solve "Fix failing parser benchmark" \
  --execution-mode hybrid_quantum \
  --privacy-mode zero_retention \
  --persistence-consent none \
  --governance-profile freedom_v1
```

## Hybrid Policy Training

```bash
# Train policy params from bootstrap dataset
python3 scripts/train_hybrid_policy.py \
  --dataset config/training/hybrid_policy_bootstrap.jsonl \
  --output config/training/hybrid_policy_trained.json \
  --report-output storage/hybrid_policy_training_report.json \
  --iterations 3000

# Continuous training for 30 minutes
python3 scripts/train_hybrid_policy.py \
  --dataset config/training/hybrid_policy_bootstrap.jsonl \
  --output config/training/hybrid_policy_trained.json \
  --report-output storage/hybrid_policy_training_report.json \
  --iterations 0 \
  --wall-clock-seconds 1800 \
  --checkpoint-every-seconds 120

# Start server with trained policy config
export HYBRID_POLICY_CONFIG_PATH=/Users/shaanp/Documents/GitHub/qAegis/services/qrms/config/training/hybrid_policy_trained.json
export GIT_COMMIT_SHA=$(git rev-parse HEAD)
cargo run --bin qrms

# Append concurrent validation log entry from artifacts
cd /Users/shaanp/Documents/GitHub/qAegis
python3 services/qrms/scripts/update_validation_log.py

# Evaluate v1 acceptance gates
python3 services/qrms/scripts/v1_acceptance_gate.py \
  services/qrms/storage/benchmark_manifest.json \
  --output services/qrms/storage/v1_acceptance_report.json

# Build internal route-labeled calibration slice when pass-delta is saturated
python3 services/qrms/scripts/build_route_labeled_slice.py \
  --manifest services/qrms/storage/benchmark_manifest.json \
  --input-tasks services/qrms/config/benchmarks/swe_bench_verified_slice_500_stub.jsonl \
  --output services/qrms/config/benchmarks/swe_bench_verified_slice_500_internal_route_labeled.jsonl

# Package claim evidence bundle
python3 services/qrms/scripts/package_v1_evidence.py \
  --benchmark-manifest services/qrms/storage/benchmark_manifest_internal_route_labeled.json \
  --acceptance-report services/qrms/storage/v1_acceptance_report_internal_route_labeled.json \
  --output-dir services/qrms/storage/evidence/v1_latest
```

## API

- REST: `http://localhost:5050/api/status`
- Hybrid solve: `POST http://localhost:5050/api/hybrid/solve`
- Hybrid execution lookup: `GET http://localhost:5050/api/hybrid/executions/:execution_id`
- Governance transparency log: `GET http://localhost:5050/api/hybrid/transparency`
- WebSocket: `ws://localhost:5050/ws`
- GUI: `http://localhost:5050`
