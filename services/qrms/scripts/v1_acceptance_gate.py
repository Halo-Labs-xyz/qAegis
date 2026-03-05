#!/usr/bin/env python3
"""Enforce v1 acceptance gates for hybrid benchmark manifests."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any, Dict, List

REQUIRED_CLAIM_KEYS = [
    "config_digest",
    "seed",
    "commit_sha",
    "task_set_digest",
    "reproducible_execution_manifest",
]


def load_manifest(path: Path) -> Dict[str, Any]:
    with path.open("r", encoding="utf-8") as handle:
        return json.load(handle)


def find_row(rows: List[Dict[str, Any]], baseline: str) -> Dict[str, Any]:
    for row in rows:
        if row.get("baseline") == baseline:
            return row
    return {}


def claim_readiness(manifest: Dict[str, Any]) -> Dict[str, Any]:
    missing = [k for k in REQUIRED_CLAIM_KEYS if manifest.get(k) in (None, "")]
    if manifest.get("commit_sha") == "unknown":
        missing.append("commit_sha(non_unknown)")
    return {
        "ok": len(missing) == 0,
        "missing": missing,
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "manifest",
        help="Path to benchmark manifest JSON",
    )
    parser.add_argument(
        "--output",
        default="services/qrms/storage/v1_acceptance_report.json",
    )
    parser.add_argument(
        "--min-pass-delta",
        type=float,
        default=0.02,
    )
    parser.add_argument(
        "--max-cost-increase",
        type=float,
        default=0.25,
    )
    args = parser.parse_args()

    manifest_path = Path(args.manifest)
    manifest = load_manifest(manifest_path)
    rows = manifest.get("results", [])

    hybrid_row = find_row(rows, "hybrid_quantum")
    classical_row = find_row(rows, "roma_classical")

    hybrid_metrics = hybrid_row.get("metrics", {})
    classical_metrics = classical_row.get("metrics", {})

    hybrid_pass = float(hybrid_metrics.get("pass_rate", 0.0))
    classical_pass = float(classical_metrics.get("pass_rate", 0.0))
    pass_delta = hybrid_pass - classical_pass

    hybrid_cost = float(hybrid_metrics.get("cost_per_task", 0.0))
    classical_cost = float(classical_metrics.get("cost_per_task", 0.0))
    if classical_cost > 0:
        cost_increase = (hybrid_cost / classical_cost) - 1.0
    else:
        cost_increase = float("inf")

    privacy_leak_rate = float(hybrid_metrics.get("privacy_leak_rate", 1.0))
    governance_pointer_coverage = float(
        hybrid_metrics.get("governance_pointer_coverage", 0.0)
    )

    checks = {
        "claims_manifest_complete": claim_readiness(manifest),
        "privacy_leak_zero": {
            "ok": privacy_leak_rate == 0.0,
            "value": privacy_leak_rate,
            "expected": 0.0,
        },
        "governance_pointer_coverage_full": {
            "ok": governance_pointer_coverage == 1.0,
            "value": governance_pointer_coverage,
            "expected": 1.0,
        },
        "hybrid_pass_delta_vs_roma_classical": {
            "ok": pass_delta >= args.min_pass_delta,
            "value": pass_delta,
            "expected_min": args.min_pass_delta,
        },
        "hybrid_cost_increase_vs_roma_classical": {
            "ok": cost_increase <= args.max_cost_increase,
            "value": cost_increase,
            "expected_max": args.max_cost_increase,
        },
    }

    overall_ok = all(
        c.get("ok", False) if isinstance(c, dict) else False for c in checks.values()
    )
    report = {
        "overall_ok": overall_ok,
        "manifest": str(manifest_path.resolve()),
        "checks": checks,
        "snapshot": {
            "commit_sha": manifest.get("commit_sha", "unknown"),
            "hybrid_pass_rate": hybrid_pass,
            "roma_classical_pass_rate": classical_pass,
            "hybrid_cost_per_task": hybrid_cost,
            "roma_classical_cost_per_task": classical_cost,
            "hybrid_privacy_leak_rate": privacy_leak_rate,
            "hybrid_governance_pointer_coverage": governance_pointer_coverage,
        },
    }

    output_path = Path(args.output)
    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(json.dumps(report, indent=2), encoding="utf-8")
    print(json.dumps(report, indent=2))
    return 0 if overall_ok else 1


if __name__ == "__main__":
    raise SystemExit(main())
