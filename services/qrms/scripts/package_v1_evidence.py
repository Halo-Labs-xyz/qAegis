#!/usr/bin/env python3
"""Package v1 training/eval artifacts into a deterministic evidence bundle."""

from __future__ import annotations

import argparse
import datetime as dt
import hashlib
import json
import shutil
from pathlib import Path
from typing import Dict


def sha256_file(path: Path) -> str:
    hasher = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            hasher.update(chunk)
    return hasher.hexdigest()


def copy_with_digest(src: Path, dst_dir: Path) -> Dict[str, str]:
    if not src.exists():
        raise FileNotFoundError(f"missing artifact: {src}")
    dst = dst_dir / src.name
    shutil.copy2(src, dst)
    return {
        "source": str(src.resolve()),
        "bundle_path": str(dst.resolve()),
        "sha256": sha256_file(dst),
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--policy-config",
        default="services/qrms/config/training/hybrid_policy_trained.json",
    )
    parser.add_argument(
        "--training-report",
        default="services/qrms/storage/hybrid_policy_training_report.json",
    )
    parser.add_argument(
        "--benchmark-manifest",
        default="services/qrms/storage/benchmark_manifest_internal_route_labeled.json",
    )
    parser.add_argument(
        "--acceptance-report",
        default="services/qrms/storage/v1_acceptance_report_internal_route_labeled.json",
    )
    parser.add_argument(
        "--validation-log",
        default="docs/CONCURRENT_VALIDATION_LOG.md",
    )
    parser.add_argument(
        "--output-dir",
        default="services/qrms/storage/evidence/v1_latest",
    )
    args = parser.parse_args()

    output_dir = Path(args.output_dir)
    output_dir.mkdir(parents=True, exist_ok=True)

    artifacts = {
        "policy_config": copy_with_digest(Path(args.policy_config), output_dir),
        "training_report": copy_with_digest(Path(args.training_report), output_dir),
        "benchmark_manifest": copy_with_digest(Path(args.benchmark_manifest), output_dir),
        "acceptance_report": copy_with_digest(Path(args.acceptance_report), output_dir),
        "validation_log": copy_with_digest(Path(args.validation_log), output_dir),
    }

    benchmark = json.loads(Path(args.benchmark_manifest).read_text(encoding="utf-8"))
    acceptance = json.loads(Path(args.acceptance_report).read_text(encoding="utf-8"))
    index = {
        "generated_at_utc": dt.datetime.now(dt.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "commit_sha": benchmark.get("commit_sha", "unknown"),
        "overall_ok": acceptance.get("overall_ok", False),
        "artifacts": artifacts,
    }

    index_path = output_dir / "EVIDENCE_INDEX.json"
    index_path.write_text(json.dumps(index, indent=2), encoding="utf-8")
    print(str(index_path.resolve()))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
