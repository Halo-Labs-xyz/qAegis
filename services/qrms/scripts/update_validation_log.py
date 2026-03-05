#!/usr/bin/env python3
"""Append a deterministic validation entry to docs/CONCURRENT_VALIDATION_LOG.md."""

from __future__ import annotations

import argparse
import datetime as dt
import json
from pathlib import Path


def load_json(path: Path) -> dict:
    with path.open("r", encoding="utf-8") as handle:
        return json.load(handle)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--manifest",
        default="services/qrms/storage/benchmark_manifest.json",
    )
    parser.add_argument(
        "--training-report",
        default="services/qrms/storage/hybrid_policy_training_report.json",
    )
    parser.add_argument(
        "--policy-config",
        default="services/qrms/config/training/hybrid_policy_trained.json",
    )
    parser.add_argument(
        "--output",
        default="docs/CONCURRENT_VALIDATION_LOG.md",
    )
    parser.add_argument(
        "--acceptance-report",
        default="services/qrms/storage/v1_acceptance_report.json",
    )
    args = parser.parse_args()

    manifest_path = Path(args.manifest)
    report_path = Path(args.training_report)
    policy_path = Path(args.policy_config)
    output_path = Path(args.output)
    acceptance_path = Path(args.acceptance_report)

    manifest = load_json(manifest_path)
    report = load_json(report_path)
    acceptance = (
        load_json(acceptance_path)
        if acceptance_path.exists()
        else {"overall_ok": "n/a", "checks": {}}
    )

    now = dt.datetime.now(dt.timezone.utc).strftime("%Y-%m-%d %H:%M:%SZ")
    rows = manifest.get("results", [])
    ok_count = sum(1 for row in rows if row.get("status") == "ok")
    total = len(rows)
    commit_sha = manifest.get("commit_sha", "unknown")
    gate_ready = "true" if commit_sha and commit_sha != "unknown" else "false"
    overview = manifest.get("overview", {})
    hybrid_row = next((r for r in rows if r.get("baseline") == "hybrid_quantum"), {})
    hybrid_metrics = hybrid_row.get("metrics", {})

    lines = [
        "",
        f"## Run {now}",
        f"- commit_sha: `{commit_sha}`",
        f"- policy_config: `{policy_path.resolve()}`",
        f"- benchmark_manifest: `{manifest_path.resolve()}`",
        f"- training_report: `{report_path.resolve()}`",
        f"- benchmark_status: `{ok_count}/{total} ok`",
        f"- claims_gate_ready: `{gate_ready}`",
        f"- best_pass_baseline: `{overview.get('best_pass_rate_baseline', 'n/a')}`",
        f"- best_pass_rate: `{overview.get('best_pass_rate', 'n/a')}`",
        f"- hybrid_pass_rate: `{hybrid_metrics.get('pass_rate', 'n/a')}`",
        f"- hybrid_cost_per_task: `{hybrid_metrics.get('cost_per_task', 'n/a')}`",
        f"- hybrid_wall_clock_per_task: `{hybrid_metrics.get('wall_clock_per_task', 'n/a')}`",
        f"- hybrid_policy_violation_rate: `{hybrid_metrics.get('policy_violation_rate', 'n/a')}`",
        f"- hybrid_privacy_leak_rate: `{hybrid_metrics.get('privacy_leak_rate', 'n/a')}`",
        f"- hybrid_fallback_rate: `{hybrid_metrics.get('fallback_rate', 'n/a')}`",
        f"- training_best_score: `{report.get('best_score', 'n/a')}`",
        f"- training_improvement: `{report.get('score_improvement', 'n/a')}`",
        f"- training_duration_seconds: `{report.get('duration_seconds', 'n/a')}`",
        f"- training_executed_iterations: `{report.get('executed_iterations', 'n/a')}`",
        f"- training_stop_reason: `{report.get('stop_reason', 'n/a')}`",
        f"- v1_acceptance_overall_ok: `{acceptance.get('overall_ok', 'n/a')}`",
    ]

    if not output_path.exists():
        output_path.write_text("# Concurrent Validation Log\n", encoding="utf-8")

    with output_path.open("a", encoding="utf-8") as handle:
        handle.write("\n".join(lines) + "\n")

    print(str(output_path.resolve()))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
