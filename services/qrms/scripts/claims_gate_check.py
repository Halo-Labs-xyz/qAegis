#!/usr/bin/env python3
"""Validate hybrid benchmark claim manifests for evidence gating."""

import argparse
import json
import sys

REQUIRED = [
    "config_digest",
    "seed",
    "commit_sha",
    "task_set_digest",
    "reproducible_execution_manifest",
]


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("manifest", help="Path to manifest JSON")
    args = parser.parse_args()

    with open(args.manifest, "r", encoding="utf-8") as f:
        data = json.load(f)

    missing = [k for k in REQUIRED if k not in data or data[k] in (None, "")]
    if not missing and data.get("commit_sha") == "unknown":
        missing.append("commit_sha(non_unknown)")
    if missing:
        print(json.dumps({"public_claim_allowed": False, "missing": missing}, indent=2))
        return 1

    print(json.dumps({"public_claim_allowed": True, "missing": []}, indent=2))
    return 0


if __name__ == "__main__":
    sys.exit(main())
