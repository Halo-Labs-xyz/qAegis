#!/usr/bin/env python3
"""Deterministic trainer for qAegis hybrid quantum policy parameters."""

from __future__ import annotations

import argparse
import hashlib
import json
import random
import time
from pathlib import Path
from typing import Any, Dict, List, Tuple

TASK_LABELS = [
    "RETRIEVE",
    "WRITE",
    "THINK",
    "CODE_INTERPRET",
    "IMAGE_GENERATION",
]
TOOL_LABELS = ["terminal", "file", "web_search", "calculator"]


def clamp(value: float, lo: float, hi: float) -> float:
    return max(lo, min(hi, value))


def normalize(values: List[float]) -> List[float]:
    total = sum(values)
    if total <= 0:
        return [1.0 / len(values)] * len(values)
    return [v / total for v in values]


def normalize_primary_weights(params: Dict[str, float]) -> None:
    s = (
        params["depth_weight"]
        + params["complexity_weight"]
        + params["dependency_weight"]
    )
    if s <= 0:
        params["depth_weight"] = 0.35
        params["complexity_weight"] = 0.40
        params["dependency_weight"] = 0.25
        return
    params["depth_weight"] /= s
    params["complexity_weight"] /= s
    params["dependency_weight"] /= s


def task_features(goal: str, depth: int, dag_nodes: int, seed: int) -> Dict[str, Any]:
    digest_input = f"{seed}:{goal}:{depth}:{dag_nodes}".encode("utf-8")
    digest = hashlib.sha256(digest_input).digest()
    complexity = min(len(goal) / 1024.0, 1.0)
    dependency_density = min(dag_nodes / 256.0, 1.0)
    depth_norm = min(depth / 16.0, 1.0)
    embedding = [complexity, dependency_density, depth_norm]
    embedding.extend([b / 255.0 for b in digest[:9]])
    return {
        "embedding": embedding,
        "digest": digest.hex(),
    }


def backend_logits(feature_digest: str, backend: str, seed: int) -> List[float]:
    logits: List[float] = []
    for class_idx in range(5):
        payload = f"{seed}:{class_idx}:{backend}:{feature_digest}".encode("utf-8")
        digest = hashlib.sha256(payload).digest()
        raw = int.from_bytes(digest[:8], byteorder="big", signed=False)
        logits.append(max(raw / float(2**64 - 1), 1e-9))
    return normalize(logits)


def classical_logits(embedding: List[float], params: Dict[str, float]) -> List[float]:
    complexity = embedding[0]
    dependency = embedding[1]
    depth = embedding[2]
    logits = [
        complexity * 0.45 + dependency * 0.40,
        complexity * 0.52,
        depth * params["depth_weight"] + (1.0 - complexity) * params["complexity_weight"],
        complexity * params["complexity_weight"]
        + dependency * params["dependency_weight"],
        (1.0 - dependency) * 0.30,
    ]
    return normalize(logits)


def apply_task_biases(logits: List[float], params: Dict[str, float]) -> List[float]:
    keys = [
        "task_bias_retrieve",
        "task_bias_write",
        "task_bias_think",
        "task_bias_code_interpret",
        "task_bias_image_generation",
    ]
    adjusted = []
    for i, value in enumerate(logits):
        bias = 1.0 + params[keys[i]]
        adjusted.append(max(value * max(bias, 0.01), 1e-9))
    return normalize(adjusted)


def tool_logits(embedding: List[float], params: Dict[str, float]) -> Dict[str, float]:
    base = sum(embedding) / max(1, len(embedding))
    return {
        "terminal": max((base * 0.92 + 0.08) * (1.0 + params["tool_bias_terminal"]), 1e-9),
        "file": max((base * 0.83 + 0.12) * (1.0 + params["tool_bias_file"]), 1e-9),
        "web_search": max(
            (base * 0.76 + 0.15) * (1.0 + params["tool_bias_web_search"]), 1e-9
        ),
        "calculator": max(
            (base * 0.65 + 0.11) * (1.0 + params["tool_bias_calculator"]), 1e-9
        ),
    }


def predict(
    row: Dict[str, Any], params: Dict[str, float], backend: str, seed: int
) -> Tuple[str, str]:
    goal = row["goal"]
    depth = int(row.get("max_depth", 1))
    dag_nodes = int(row.get("dag_nodes", 1))
    feat = task_features(goal, depth, dag_nodes, seed)
    q_logits = backend_logits(feat["digest"], backend, seed)
    c_logits = classical_logits(feat["embedding"], params)
    backend_weight = params["backend_weight"]
    blended = [
        q_logits[i] * backend_weight + c_logits[i] * (1.0 - backend_weight)
        for i in range(5)
    ]
    task_scores = apply_task_biases(blended, params)
    task_label = TASK_LABELS[max(range(5), key=lambda i: task_scores[i])]
    tool_scores = tool_logits(feat["embedding"], params)
    tool_label = max(tool_scores, key=tool_scores.get)
    return task_label, tool_label


def default_params() -> Dict[str, float]:
    return {
        "depth_weight": 0.35,
        "complexity_weight": 0.40,
        "dependency_weight": 0.25,
        "backend_weight": 0.70,
        "task_bias_retrieve": 0.0,
        "task_bias_write": 0.0,
        "task_bias_think": 0.0,
        "task_bias_code_interpret": 0.0,
        "task_bias_image_generation": 0.0,
        "tool_bias_terminal": 0.0,
        "tool_bias_file": 0.0,
        "tool_bias_web_search": 0.0,
        "tool_bias_calculator": 0.0,
    }


def random_candidate(rng: random.Random) -> Dict[str, float]:
    params = {
        "depth_weight": rng.uniform(0.05, 0.90),
        "complexity_weight": rng.uniform(0.05, 0.90),
        "dependency_weight": rng.uniform(0.05, 0.90),
        "backend_weight": rng.uniform(0.30, 0.95),
        "task_bias_retrieve": rng.uniform(-0.35, 0.35),
        "task_bias_write": rng.uniform(-0.35, 0.35),
        "task_bias_think": rng.uniform(-0.35, 0.35),
        "task_bias_code_interpret": rng.uniform(-0.35, 0.35),
        "task_bias_image_generation": rng.uniform(-0.35, 0.35),
        "tool_bias_terminal": rng.uniform(-0.35, 0.35),
        "tool_bias_file": rng.uniform(-0.35, 0.35),
        "tool_bias_web_search": rng.uniform(-0.35, 0.35),
        "tool_bias_calculator": rng.uniform(-0.35, 0.35),
    }
    normalize_primary_weights(params)
    return params


def mutate_candidate(best: Dict[str, float], rng: random.Random) -> Dict[str, float]:
    params = dict(best)
    params["depth_weight"] = clamp(params["depth_weight"] + rng.gauss(0, 0.08), 0.01, 1.0)
    params["complexity_weight"] = clamp(
        params["complexity_weight"] + rng.gauss(0, 0.08), 0.01, 1.0
    )
    params["dependency_weight"] = clamp(
        params["dependency_weight"] + rng.gauss(0, 0.08), 0.01, 1.0
    )
    params["backend_weight"] = clamp(
        params["backend_weight"] + rng.gauss(0, 0.05), 0.0, 1.0
    )
    for key in [
        "task_bias_retrieve",
        "task_bias_write",
        "task_bias_think",
        "task_bias_code_interpret",
        "task_bias_image_generation",
        "tool_bias_terminal",
        "tool_bias_file",
        "tool_bias_web_search",
        "tool_bias_calculator",
    ]:
        params[key] = clamp(params[key] + rng.gauss(0, 0.04), -0.95, 2.0)
    normalize_primary_weights(params)
    return params


def score_dataset(
    rows: List[Dict[str, Any]], params: Dict[str, float], backend: str, seed: int
) -> float:
    if not rows:
        return 0.0
    total = 0.0
    for row in rows:
        pred_task, pred_tool = predict(row, params, backend, seed)
        target_task = row["target_task_type"]
        target_tool = row.get("target_tool")
        task_ok = 1.0 if pred_task == target_task else 0.0
        if target_tool:
            tool_ok = 1.0 if pred_tool == target_tool else 0.0
            total += 0.75 * task_ok + 0.25 * tool_ok
        else:
            total += task_ok
    return total / len(rows)


def load_dataset(path: Path) -> List[Dict[str, Any]]:
    rows: List[Dict[str, Any]] = []
    with path.open("r", encoding="utf-8") as handle:
        for line_idx, line in enumerate(handle, start=1):
            stripped = line.strip()
            if not stripped or stripped.startswith("#"):
                continue
            row = json.loads(stripped)
            if "goal" not in row or "target_task_type" not in row:
                raise ValueError(f"Invalid dataset row {line_idx}: missing required keys")
            if row["target_task_type"] not in TASK_LABELS:
                raise ValueError(
                    f"Invalid target_task_type at row {line_idx}: {row['target_task_type']}"
                )
            if row.get("target_tool") and row["target_tool"] not in TOOL_LABELS:
                raise ValueError(
                    f"Invalid target_tool at row {line_idx}: {row['target_tool']}"
                )
            rows.append(row)
    return rows


def write_config(path: Path, config: Dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(config, indent=2), encoding="utf-8")


def write_report(path: Path, report: Dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(report, indent=2), encoding="utf-8")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--dataset",
        default="services/qrms/config/training/hybrid_policy_bootstrap.jsonl",
    )
    parser.add_argument(
        "--output",
        default="services/qrms/config/training/hybrid_policy_trained.json",
    )
    parser.add_argument(
        "--report-output",
        default="services/qrms/storage/hybrid_policy_training_report.json",
    )
    parser.add_argument("--backend", default="lightning.qubit")
    parser.add_argument("--seed", type=int, default=1337)
    parser.add_argument("--timeout-ms", type=int, default=1200)
    parser.add_argument("--iterations", type=int, default=3000)
    parser.add_argument("--wall-clock-seconds", type=int, default=0)
    parser.add_argument("--checkpoint-every-seconds", type=int, default=120)
    args = parser.parse_args()

    dataset_path = Path(args.dataset)
    rows = load_dataset(dataset_path)
    if not rows:
        raise RuntimeError("Dataset is empty")
    if args.iterations <= 0 and args.wall_clock_seconds <= 0:
        raise RuntimeError("set --iterations > 0 or --wall-clock-seconds > 0")

    rng = random.Random(args.seed)
    best = default_params()
    normalize_primary_weights(best)
    baseline_score = score_dataset(rows, best, args.backend, args.seed)
    best_score = baseline_score
    output_path = Path(args.output)
    report_path = Path(args.report_output)
    train_started = time.monotonic()
    next_checkpoint = train_started + max(args.checkpoint_every_seconds, 1)
    deadline = (
        train_started + args.wall_clock_seconds if args.wall_clock_seconds > 0 else None
    )
    iteration_limit = args.iterations if args.iterations > 0 else None
    warmup_iterations = max(128, (iteration_limit // 3) if iteration_limit else 1000)
    executed_iterations = 0
    stop_reason = "completed_iteration_budget"

    while True:
        now = time.monotonic()
        if iteration_limit is not None and executed_iterations >= iteration_limit:
            stop_reason = "completed_iteration_budget"
            break
        if deadline is not None and now >= deadline:
            stop_reason = "reached_wall_clock_limit"
            break

        candidate = (
            random_candidate(rng)
            if executed_iterations < warmup_iterations
            else mutate_candidate(best, rng)
        )
        candidate_score = score_dataset(rows, candidate, args.backend, args.seed)
        if candidate_score > best_score:
            best = candidate
            best_score = candidate_score

        executed_iterations += 1
        if args.checkpoint_every_seconds > 0 and now >= next_checkpoint:
            checkpoint_config = {
                "backend": args.backend,
                "seed": args.seed,
                "timeout_ms": max(1, args.timeout_ms),
                "policy_params": best,
            }
            write_config(output_path, checkpoint_config)
            next_checkpoint = now + args.checkpoint_every_seconds

    config = {
        "backend": args.backend,
        "seed": args.seed,
        "timeout_ms": max(1, args.timeout_ms),
        "policy_params": best,
    }
    write_config(output_path, config)

    duration_seconds = time.monotonic() - train_started
    report = {
        "trained_at_unix": int(time.time()),
        "dataset": str(dataset_path.resolve()),
        "iterations": args.iterations,
        "wall_clock_seconds": args.wall_clock_seconds,
        "executed_iterations": executed_iterations,
        "duration_seconds": duration_seconds,
        "stop_reason": stop_reason,
        "sample_count": len(rows),
        "baseline_score": baseline_score,
        "best_score": best_score,
        "score_improvement": best_score - baseline_score,
        "output_config": str(output_path.resolve()),
    }
    write_report(report_path, report)

    print(str(output_path.resolve()))
    print(str(report_path.resolve()))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
