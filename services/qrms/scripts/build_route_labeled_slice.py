#!/usr/bin/env python3
"""Build an internal route-labeled benchmark slice from hybrid benchmark runs."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any, Dict, List


def load_json(path: Path) -> Dict[str, Any]:
    with path.open("r", encoding="utf-8") as handle:
        return json.load(handle)


def load_jsonl(path: Path) -> List[Dict[str, Any]]:
    rows: List[Dict[str, Any]] = []
    with path.open("r", encoding="utf-8") as handle:
        for idx, line in enumerate(handle, start=1):
            stripped = line.strip()
            if not stripped or stripped.startswith("#"):
                continue
            row = json.loads(stripped)
            if "task_id" not in row or "goal" not in row:
                raise ValueError(f"invalid task row {idx}: requires task_id and goal")
            rows.append(row)
    return rows


def write_jsonl(path: Path, rows: List[Dict[str, Any]]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8") as handle:
        for row in rows:
            handle.write(json.dumps(row, separators=(",", ":")) + "\n")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--manifest",
        default="services/qrms/storage/benchmark_manifest.json",
    )
    parser.add_argument(
        "--input-tasks",
        default="services/qrms/config/benchmarks/swe_bench_verified_slice_500_stub.jsonl",
    )
    parser.add_argument(
        "--output",
        default="services/qrms/config/benchmarks/swe_bench_verified_slice_500_internal_route_labeled.jsonl",
    )
    args = parser.parse_args()

    manifest = load_json(Path(args.manifest))
    baselines = manifest.get("results", [])
    hybrid = next((row for row in baselines if row.get("baseline") == "hybrid_quantum"), None)
    if not hybrid:
        raise RuntimeError("hybrid_quantum baseline not found in manifest")

    route_map: Dict[str, Dict[str, str]] = {}
    for run in hybrid.get("task_runs", []):
        task_id = run.get("task_id")
        if not task_id:
            continue
        task_type = run.get("routed_task_type")
        tool = run.get("routed_tool")
        if task_type and tool:
            route_map[str(task_id)] = {
                "expected_task_type": str(task_type),
                "expected_tool": str(tool),
            }

    tasks = load_jsonl(Path(args.input_tasks))
    out_rows: List[Dict[str, Any]] = []
    updated = 0
    for row in tasks:
        task_id = str(row["task_id"])
        cloned = dict(row)
        metadata = dict(cloned.get("metadata", {}))
        metadata["slice_kind"] = "internal_route_labeled"
        cloned["metadata"] = metadata
        if task_id in route_map:
            cloned["expected_task_type"] = route_map[task_id]["expected_task_type"]
            cloned["expected_tool"] = route_map[task_id]["expected_tool"]
            updated += 1
        out_rows.append(cloned)

    output_path = Path(args.output)
    write_jsonl(output_path, out_rows)

    summary = {
        "output": str(output_path.resolve()),
        "total_rows": len(out_rows),
        "route_labeled_rows": updated,
    }
    print(json.dumps(summary, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
