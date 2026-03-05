# Concurrent Validation Log

## 2026-03-04
- Unit test modules executed:
  - `hybrid_quantum::tests::`
  - `security_plane::tests::`
  - `governance_plane::tests::`
  - `hybrid_engine::tests::`
- API checks:
  - `POST /api/hybrid/solve`
  - `GET /api/hybrid/executions/:execution_id`
  - `GET /api/hybrid/transparency`
- Benchmark + claims checks:
  - `services/qrms/scripts/benchmark_runner.py`
  - `services/qrms/scripts/claims_gate_check.py`

## Logging Rule
For every training/eval cycle, append:
- commit SHA
- policy config path
- benchmark manifest path
- claims gate result
- policy violation/leak outcomes

## Run 2026-03-04 23:00:17Z
- commit_sha: `a1aa758bcdffad8e7bc22fe2be6299ca1f81d13f`
- policy_config: `/Users/shaanp/Documents/GitHub/qAegis/services/qrms/config/training/hybrid_policy_trained.json`
- benchmark_manifest: `/Users/shaanp/Documents/GitHub/qAegis/services/qrms/storage/benchmark_manifest.json`
- training_report: `/Users/shaanp/Documents/GitHub/qAegis/services/qrms/storage/hybrid_policy_training_report.json`
- benchmark_status: `5/5 ok`
- claims_gate_ready: `true`
- training_best_score: `0.4375`
- training_improvement: `0.3`

## Run 2026-03-04 23:03:18Z
- commit_sha: `a1aa758bcdffad8e7bc22fe2be6299ca1f81d13f`
- policy_config: `/Users/shaanp/Documents/GitHub/qAegis/services/qrms/config/training/hybrid_policy_trained.json`
- benchmark_manifest: `/Users/shaanp/Documents/GitHub/qAegis/services/qrms/storage/benchmark_manifest.json`
- training_report: `/Users/shaanp/Documents/GitHub/qAegis/services/qrms/storage/hybrid_policy_training_report.json`
- benchmark_status: `5/5 ok`
- claims_gate_ready: `true`
- training_best_score: `0.4375`
- training_improvement: `0.3`

## Run 2026-03-04 23:10:47Z
- commit_sha: `a1aa758bcdffad8e7bc22fe2be6299ca1f81d13f`
- policy_config: `/Users/shaanp/Documents/GitHub/qAegis/services/qrms/config/training/hybrid_policy_trained.json`
- benchmark_manifest: `/Users/shaanp/Documents/GitHub/qAegis/services/qrms/storage/benchmark_manifest.json`
- training_report: `/Users/shaanp/Documents/GitHub/qAegis/services/qrms/storage/hybrid_policy_training_report.json`
- benchmark_status: `5/5 ok`
- claims_gate_ready: `true`
- best_pass_baseline: `swe_bench/codex`
- best_pass_rate: `1.0`
- hybrid_pass_rate: `1.0`
- hybrid_cost_per_task: `0.30876000000000003`
- hybrid_wall_clock_per_task: `0.01941514015197754`
- hybrid_policy_violation_rate: `0.0`
- hybrid_privacy_leak_rate: `0.0`
- hybrid_fallback_rate: `0.0`
- training_best_score: `0.4375`
- training_improvement: `0.3`
- training_duration_seconds: `3.0009636250324547`
- training_executed_iterations: `10085`
- training_stop_reason: `reached_wall_clock_limit`

## Run 2026-03-05 00:14:16Z
- commit_sha: `a1aa758bcdffad8e7bc22fe2be6299ca1f81d13f`
- policy_config: `/Users/shaanp/Documents/GitHub/qAegis/services/qrms/config/training/hybrid_policy_trained.json`
- benchmark_manifest: `/Users/shaanp/Documents/GitHub/qAegis/services/qrms/storage/benchmark_manifest.json`
- training_report: `/Users/shaanp/Documents/GitHub/qAegis/services/qrms/storage/hybrid_policy_training_report.json`
- benchmark_status: `5/5 ok`
- claims_gate_ready: `true`
- best_pass_baseline: `swe_bench/codex`
- best_pass_rate: `1.0`
- hybrid_pass_rate: `1.0`
- hybrid_cost_per_task: `0.30876000000000003`
- hybrid_wall_clock_per_task: `0.01989912986755371`
- hybrid_policy_violation_rate: `0.0`
- hybrid_privacy_leak_rate: `0.0`
- hybrid_fallback_rate: `0.0`
- training_best_score: `0.4375`
- training_improvement: `0.3`
- training_duration_seconds: `3.0009636250324547`
- training_executed_iterations: `10085`
- training_stop_reason: `reached_wall_clock_limit`

## Run 2026-03-05 00:44:46Z
- commit_sha: `a1aa758bcdffad8e7bc22fe2be6299ca1f81d13f`
- policy_config: `/Users/shaanp/Documents/GitHub/qAegis/services/qrms/config/training/hybrid_policy_trained.json`
- benchmark_manifest: `/Users/shaanp/Documents/GitHub/qAegis/services/qrms/storage/benchmark_manifest.json`
- training_report: `/Users/shaanp/Documents/GitHub/qAegis/services/qrms/storage/hybrid_policy_training_report.json`
- benchmark_status: `5/5 ok`
- claims_gate_ready: `true`
- best_pass_baseline: `swe_bench/codex`
- best_pass_rate: `1.0`
- hybrid_pass_rate: `1.0`
- hybrid_cost_per_task: `0.30779999999999996`
- hybrid_wall_clock_per_task: `0.02234787940979004`
- hybrid_policy_violation_rate: `0.0`
- hybrid_privacy_leak_rate: `0.0`
- hybrid_fallback_rate: `0.0`
- training_best_score: `0.4625`
- training_improvement: `0.325`
- training_duration_seconds: `1800.0006894999533`
- training_executed_iterations: `6288970`
- training_stop_reason: `reached_wall_clock_limit`

## Run 2026-03-05 00:49:02Z
- commit_sha: `a1aa758bcdffad8e7bc22fe2be6299ca1f81d13f`
- policy_config: `/Users/shaanp/Documents/GitHub/qAegis/services/qrms/config/training/hybrid_policy_trained.json`
- benchmark_manifest: `/Users/shaanp/Documents/GitHub/qAegis/services/qrms/storage/benchmark_manifest.json`
- training_report: `/Users/shaanp/Documents/GitHub/qAegis/services/qrms/storage/hybrid_policy_training_report.json`
- benchmark_status: `5/5 ok`
- claims_gate_ready: `true`
- best_pass_baseline: `swe_bench/codex`
- best_pass_rate: `1.0`
- hybrid_pass_rate: `1.0`
- hybrid_cost_per_task: `0.30779999999999996`
- hybrid_wall_clock_per_task: `0.020362210273742676`
- hybrid_policy_violation_rate: `0.0`
- hybrid_privacy_leak_rate: `0.0`
- hybrid_fallback_rate: `0.0`
- training_best_score: `0.4625`
- training_improvement: `0.325`
- training_duration_seconds: `1800.0006894999533`
- training_executed_iterations: `6288970`
- training_stop_reason: `reached_wall_clock_limit`
- v1_acceptance_overall_ok: `False`

## Run 2026-03-05 00:52:15Z
- commit_sha: `a1aa758bcdffad8e7bc22fe2be6299ca1f81d13f`
- policy_config: `/Users/shaanp/Documents/GitHub/qAegis/services/qrms/config/training/hybrid_policy_trained.json`
- benchmark_manifest: `/Users/shaanp/Documents/GitHub/qAegis/services/qrms/storage/benchmark_manifest_internal_route_labeled.json`
- training_report: `/Users/shaanp/Documents/GitHub/qAegis/services/qrms/storage/hybrid_policy_training_report.json`
- benchmark_status: `5/5 ok`
- claims_gate_ready: `true`
- best_pass_baseline: `hybrid_quantum`
- best_pass_rate: `0.3`
- hybrid_pass_rate: `0.3`
- hybrid_cost_per_task: `0.30977999999999994`
- hybrid_wall_clock_per_task: `0.020908188819885255`
- hybrid_policy_violation_rate: `0.0`
- hybrid_privacy_leak_rate: `0.0`
- hybrid_fallback_rate: `0.0`
- training_best_score: `0.4625`
- training_improvement: `0.325`
- training_duration_seconds: `1800.0006894999533`
- training_executed_iterations: `6288970`
- training_stop_reason: `reached_wall_clock_limit`
- v1_acceptance_overall_ok: `True`

## Run 2026-03-05 01:04:45Z
- commit_sha: `a1aa758bcdffad8e7bc22fe2be6299ca1f81d13f`
- policy_config: `/Users/shaanp/Documents/GitHub/qAegis/services/qrms/config/training/hybrid_policy_trained.json`
- benchmark_manifest: `/Users/shaanp/Documents/GitHub/qAegis/services/qrms/storage/benchmark_manifest_internal_route_labeled.json`
- training_report: `/Users/shaanp/Documents/GitHub/qAegis/services/qrms/storage/hybrid_policy_training_report.json`
- benchmark_status: `5/5 ok`
- claims_gate_ready: `true`
- best_pass_baseline: `hybrid_quantum`
- best_pass_rate: `0.3`
- hybrid_pass_rate: `0.3`
- hybrid_cost_per_task: `0.30977999999999994`
- hybrid_wall_clock_per_task: `0.022548353672027587`
- hybrid_policy_violation_rate: `0.0`
- hybrid_privacy_leak_rate: `0.0`
- hybrid_fallback_rate: `0.0`
- training_best_score: `0.4625`
- training_improvement: `0.325`
- training_duration_seconds: `1800.0006894999533`
- training_executed_iterations: `6288970`
- training_stop_reason: `reached_wall_clock_limit`
- v1_acceptance_overall_ok: `True`
