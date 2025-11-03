#!/usr/bin/env python3

import json
import math
import pathlib
import sys
from typing import Dict, Tuple


def load_estimates(path: pathlib.Path) -> Dict[str, float]:
    with path.open("r", encoding="utf-8") as handle:
        data = json.load(handle)
    try:
        benches = data["benchmarks"]
    except KeyError as exc:
        raise SystemExit(f"{path}: missing 'benchmarks' key") from exc

    means: Dict[str, float] = {}
    for name, entry in benches.items():
        try:
            mean = entry["criterion_estimates_v1"]["mean"]["point_estimate"]
        except KeyError as exc:
            raise SystemExit(f"{path}: missing mean for benchmark '{name}'") from exc
        means[name] = float(mean)
    return means


def main() -> int:
    if len(sys.argv) < 3 or len(sys.argv) > 4:
        print(
            "usage: compare_benchmarks.py <baseline.json> <candidate.json> [threshold_percent]",
            file=sys.stderr,
        )
        return 2

    baseline_path = pathlib.Path(sys.argv[1])
    candidate_path = pathlib.Path(sys.argv[2])
    threshold = float(sys.argv[3]) if len(sys.argv) == 4 else 5.0

    baseline = load_estimates(baseline_path)
    candidate = load_estimates(candidate_path)

    missing = sorted(set(baseline) - set(candidate))
    if missing:
        for name in missing:
            print(f"missing benchmark in candidate: {name}")
        return 1

    regressions: Dict[str, Tuple[float, float, float]] = {}

    print("Benchmark comparison (ratios > 1.0 are slower):")
    for name in sorted(baseline):
        base_mean = baseline[name]
        cand_mean = candidate.get(name)
        if cand_mean is None:
            continue  # already handled missing above

        ratio = cand_mean / base_mean if base_mean else math.inf
        diff_pct = (ratio - 1.0) * 100.0
        status = ""
        if diff_pct > threshold:
            status = "REGRESSION"
            regressions[name] = (ratio, diff_pct, cand_mean)
        elif diff_pct < -threshold:
            status = "IMPROVEMENT"
        print(
            f"  {name:<28} {ratio:6.3f}x  ({diff_pct:+7.2f}%)  baseline={base_mean:.3f}  candidate={cand_mean:.3f} {status}"
        )

    if regressions:
        print("\nDetected regressions over baseline (threshold {:.2f}%):".format(threshold))
        for name, (ratio, diff_pct, cand_mean) in regressions.items():
            print(f"  {name}: {ratio:.3f}x slower (+{diff_pct:.2f}%), candidate mean {cand_mean:.3f}")
        return 1

    print("\nNo regressions detected.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
