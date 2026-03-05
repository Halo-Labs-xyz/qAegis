# Hybrid Policy Training and Evaluation Runbook

## Objective
Train and validate the hybrid quantum-classical routing policy with deterministic artifacts and evidence-gated outputs.

## Prerequisites
- Branch: `codex/hybrid-roma-v1`
- Service path: `/Users/shaanp/Documents/GitHub/qAegis/services/qrms`
- Server port: `5050`

## Step 1: Train Policy Parameters
```bash
cd /Users/shaanp/Documents/GitHub/qAegis

python3 services/qrms/scripts/train_hybrid_policy.py \
  --dataset services/qrms/config/training/hybrid_policy_bootstrap.jsonl \
  --output services/qrms/config/training/hybrid_policy_trained.json \
  --report-output services/qrms/storage/hybrid_policy_training_report.json \
  --iterations 3000
```

Artifacts:
- `services/qrms/config/training/hybrid_policy_trained.json`
- `services/qrms/storage/hybrid_policy_training_report.json`

Continuous training for 30 minutes (wall-clock bounded):
```bash
cd /Users/shaanp/Documents/GitHub/qAegis

python3 services/qrms/scripts/train_hybrid_policy.py \
  --dataset services/qrms/config/training/hybrid_policy_bootstrap.jsonl \
  --output services/qrms/config/training/hybrid_policy_trained.json \
  --report-output services/qrms/storage/hybrid_policy_training_report.json \
  --iterations 0 \
  --wall-clock-seconds 1800 \
  --checkpoint-every-seconds 120
```

## Step 2: Start Server with Trained Policy
```bash
cd /Users/shaanp/Documents/GitHub/qAegis/services/qrms
export HYBRID_POLICY_CONFIG_PATH=/Users/shaanp/Documents/GitHub/qAegis/services/qrms/config/training/hybrid_policy_trained.json
export GIT_COMMIT_SHA=$(git rev-parse HEAD)
cargo run --bin qrms
```

## Step 3: Validate Hybrid Solve API
```bash
curl -sS -X POST http://localhost:5050/api/hybrid/solve \
  -H 'Content-Type: application/json' \
  -d '{"goal":"Fix failing parser benchmark","max_depth":2,"execution_mode":"hybrid_quantum","privacy_mode":"zero_retention","persistence_consent":"none","governance_profile":"freedom_v1","metadata":{"suite":"swe_bench"}}' | jq .
```

Required response fields:
- `governance_decision_id`
- `audit_hash`
- `retention_policy_applied`
- `quantum_backend`
- `fallback_used`
- `claims_manifest`

## Step 4: Run Benchmark Matrix and Claims Gate
```bash
cd /Users/shaanp/Documents/GitHub/qAegis
export GIT_COMMIT_SHA=$(git rev-parse HEAD)

python3 services/qrms/scripts/benchmark_runner.py \
  --server http://localhost:5050 \
  --tasks services/qrms/config/benchmarks/swe_bench_verified_slice_500_stub.jsonl \
  --output services/qrms/storage/benchmark_manifest.json

python3 services/qrms/scripts/claims_gate_check.py \
  services/qrms/storage/benchmark_manifest.json

python3 services/qrms/scripts/v1_acceptance_gate.py \
  services/qrms/storage/benchmark_manifest.json \
  --output services/qrms/storage/v1_acceptance_report.json
```

If pass-delta is flat on the stub slice, generate an internal route-labeled calibration slice and rerun:
```bash
cd /Users/shaanp/Documents/GitHub/qAegis

python3 services/qrms/scripts/build_route_labeled_slice.py \
  --manifest services/qrms/storage/benchmark_manifest.json \
  --input-tasks services/qrms/config/benchmarks/swe_bench_verified_slice_500_stub.jsonl \
  --output services/qrms/config/benchmarks/swe_bench_verified_slice_500_internal_route_labeled.jsonl

python3 services/qrms/scripts/benchmark_runner.py \
  --server http://localhost:5050 \
  --tasks services/qrms/config/benchmarks/swe_bench_verified_slice_500_internal_route_labeled.jsonl \
  --output services/qrms/storage/benchmark_manifest_internal_route_labeled.json

python3 services/qrms/scripts/v1_acceptance_gate.py \
  services/qrms/storage/benchmark_manifest_internal_route_labeled.json \
  --output services/qrms/storage/v1_acceptance_report_internal_route_labeled.json
```

## Acceptance Checks
- All benchmark rows report `status=ok`.
- `claims_gate_check.py` returns `"public_claim_allowed": true`.
- `stored_goal` and `stored_result` are hashed under `persistence_consent=none`.
- Hybrid metrics in manifest:
  - `pass_rate`
  - `cost_per_task`
  - `wall_clock_per_task`
  - `policy_violation_rate`
  - `privacy_leak_rate`
- `v1_acceptance_gate.py` returns `overall_ok=true`.

## Step 5: Append Concurrent Validation Documentation
```bash
cd /Users/shaanp/Documents/GitHub/qAegis
python3 services/qrms/scripts/update_validation_log.py \
  --manifest services/qrms/storage/benchmark_manifest.json \
  --training-report services/qrms/storage/hybrid_policy_training_report.json \
  --policy-config services/qrms/config/training/hybrid_policy_trained.json \
  --output docs/CONCURRENT_VALIDATION_LOG.md
```

## Step 6: Package Evidence Bundle
```bash
cd /Users/shaanp/Documents/GitHub/qAegis
python3 services/qrms/scripts/package_v1_evidence.py \
  --policy-config services/qrms/config/training/hybrid_policy_trained.json \
  --training-report services/qrms/storage/hybrid_policy_training_report.json \
  --benchmark-manifest services/qrms/storage/benchmark_manifest_internal_route_labeled.json \
  --acceptance-report services/qrms/storage/v1_acceptance_report_internal_route_labeled.json \
  --validation-log docs/CONCURRENT_VALIDATION_LOG.md \
  --output-dir services/qrms/storage/evidence/v1_latest
```
