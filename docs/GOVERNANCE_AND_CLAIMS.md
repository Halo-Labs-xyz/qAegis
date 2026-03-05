# Governance and Claims Gate (qAegis v1)

## Governance Flow
Execution output is adjudicated before release.

1. Run hybrid/classical execution.
2. Evaluate constitution policy (`freedom_v1`).
3. Create governance decision with deterministic rationale class.
4. Append decision to transparency log with hash-chain integrity.
5. Return response metadata:
   - `governance_decision_id`
   - `audit_hash`
   - `retention_policy_applied`

## Components
- `services/qrms/src/governance_plane.rs`
  - `Constitution`
  - `GovernanceAdjudicator`
  - `AuditLog` (append-only hash chain)
- `services/qrms/src/hybrid_engine.rs`
  - claims manifest generation + gate result

## Claims Gate
A run is claim-ready only when all fields exist and are non-empty:
- `config_digest`
- `seed`
- `commit_sha`
- `task_set_digest`
- `reproducible_execution_manifest`

Additional enforcement:
- `commit_sha=unknown` is treated as not claim-ready.
- Set `GIT_COMMIT_SHA=$(git rev-parse HEAD)` before starting `qrms` so runtime responses carry a non-unknown commit SHA.

Validation helper:
- `services/qrms/scripts/claims_gate_check.py`
- `services/qrms/scripts/v1_acceptance_gate.py` (checks claims + privacy + governance coverage + pass/cost thresholds)
- `services/qrms/scripts/package_v1_evidence.py` (copies artifacts and emits `EVIDENCE_INDEX.json` with SHA-256 checksums)
