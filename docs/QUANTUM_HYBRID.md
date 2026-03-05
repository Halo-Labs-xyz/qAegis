# Quantum-Hybrid Execution (qAegis v1)

## Scope
qAegis implements an opt-in `hybrid_quantum` execution path in `services/qrms` for software-engineering benchmark workloads.

## Components
- `services/qrms/src/hybrid_quantum.rs`
  - `QuantumBackend` interface
  - `PennyLaneSimulatorBackend` abstraction (`lightning.qubit` backend label)
  - deterministic feature extraction from goal/depth/DAG-size context
  - routing logits for task-type and tool selection
  - trainable blend+bias parameters (`backend_weight`, task/tool biases)
  - automatic classical fallback on backend failure/timeout
- `services/qrms/src/hybrid_engine.rs`
  - orchestration of routing, execution, governance, retention, claims manifest generation
  - runtime config load from `HYBRID_POLICY_CONFIG_PATH`

## API
`POST /api/hybrid/solve`
- request fields: `execution_mode`, `privacy_mode`, `persistence_consent`, `governance_profile`
- response fields: `governance_decision_id`, `audit_hash`, `retention_policy_applied`, `quantum_backend`, `fallback_used`, `claims_manifest`

## Reproducibility
- fixed default seed: `1337`
- serialized policy params in `QuantumConfig`
- manifest includes: `config_digest`, `seed`, `commit_sha`, `task_set_digest`, `reproducible_execution_manifest`
- `public_claim_allowed=false` when `commit_sha` is missing/`unknown`

## Training Workflow
1. Prepare dataset (`jsonl`) with fields:
   - `goal`
   - `max_depth`
   - `dag_nodes`
   - `target_task_type`
   - optional `target_tool`
2. Train deterministic parameters:
   - `python3 services/qrms/scripts/train_hybrid_policy.py --dataset services/qrms/config/training/hybrid_policy_bootstrap.jsonl --output services/qrms/config/training/hybrid_policy_trained.json`
3. Train continuously for 30 minutes:
   - `python3 services/qrms/scripts/train_hybrid_policy.py --dataset services/qrms/config/training/hybrid_policy_bootstrap.jsonl --output services/qrms/config/training/hybrid_policy_trained.json --report-output services/qrms/storage/hybrid_policy_training_report.json --iterations 0 --wall-clock-seconds 1800 --checkpoint-every-seconds 120`
4. Start service with trained config:
   - `HYBRID_POLICY_CONFIG_PATH=services/qrms/config/training/hybrid_policy_trained.json cargo run --bin qrms`

## Profile
`services/qrms/config/profiles/swe_bench/hybrid_quantum.yaml` defines benchmark mode, baselines, and claims gate requirements.

## Internal Calibration Slice
- `services/qrms/scripts/build_route_labeled_slice.py` derives expected route labels from `hybrid_quantum` task runs for internal pass-delta calibration.
- Use only for internal routing calibration, not public performance claims.
