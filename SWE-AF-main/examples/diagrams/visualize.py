# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
"""
Pipeline BI Visualization — parses agent execution logs, computes chart
datasets in pure Python, then hands off to a Deno+D3 renderer that
produces 14 presentation-quality SVG charts.

Usage:  uv run visualize.py
        python3 visualize.py
"""

from __future__ import annotations

import json
import subprocess
import sys
import tempfile
from collections import defaultdict
from pathlib import Path

# ── paths ──────────────────────────────────────────────────────────────
SCRIPT_DIR = Path(__file__).resolve().parent
LOGS_DIR = SCRIPT_DIR / ".artifacts" / "logs"
CHARTS_DIR = SCRIPT_DIR / "charts"
CHARTS_DIR.mkdir(exist_ok=True)

# ── style ──────────────────────────────────────────────────────────────
CATEGORY_PALETTE = {
    "product_manager": "#4e79a7",
    "architect": "#59a14f",
    "sprint_planner": "#9c755f",
    "tech_lead": "#76b7b2",
    "issue_writer": "#edc948",
    "coder": "#f28e2b",
    "reviewer": "#e15759",
    "qa": "#b07aa1",
    "synthesizer": "#ff9da7",
    "merger": "#bab0ac",
    "integration_tester": "#86bcb6",
    "workspace_setup": "#a0cbe8",
    "workspace_cleanup": "#d4a6c8",
}
PHASE_ORDER = [
    "product_manager",
    "architect",
    "sprint_planner",
    "tech_lead",
    "issue_writer",
    "workspace_setup",
    "coder",
    "reviewer",
    "qa",
    "synthesizer",
    "merger",
    "integration_tester",
    "workspace_cleanup",
]


# ── data parsing (pure stdlib) ────────────────────────────────────────
def parse_logs() -> list[dict]:
    """Parse all *.jsonl in logs dir, return one dict per agent execution."""
    rows: list[dict] = []
    for path in sorted(LOGS_DIR.glob("*.jsonl")):
        fname = path.stem
        events: list[dict] = []
        for line in path.read_text().splitlines():
            if line.strip():
                events.append(json.loads(line))

        start_ev = next((e for e in events if e.get("event") == "start"), None)
        result_ev = next((e for e in events if e.get("event") == "result"), None)
        end_ev = next((e for e in events if e.get("event") == "end"), None)
        if not start_ev or not end_ev:
            continue

        ts_start = start_ev["ts"]
        ts_end = end_ev["ts"]
        cost = end_ev.get("cost_usd", 0.0)
        num_turns = end_ev.get("num_turns", 0)
        duration_ms = (
            result_ev.get("duration_ms", (ts_end - ts_start) * 1000)
            if result_ev
            else (ts_end - ts_start) * 1000
        )

        tool_calls = 0
        text_chars = 0
        for ev in events:
            if ev.get("event") == "assistant":
                for block in ev.get("content", []):
                    if block.get("type") == "tool_use":
                        tool_calls += 1
                    elif block.get("type") == "text":
                        text_chars += len(block.get("text", ""))

        category, issue, iteration = _parse_filename(fname)

        rows.append(
            {
                "filename": fname,
                "category": category,
                "issue": issue,
                "iteration": iteration,
                "ts_start": ts_start,
                "ts_end": ts_end,
                "cost_usd": cost,
                "duration_s": duration_ms / 1000,
                "num_turns": num_turns,
                "tool_calls": tool_calls,
                "text_chars": text_chars,
                "is_error": end_ev.get("is_error", False),
                "model": start_ev.get("model", "unknown"),
            }
        )

    rows.sort(key=lambda r: r["ts_start"])
    return rows


def _parse_filename(fname: str) -> tuple[str, str, int]:
    if fname in ("architect", "product_manager", "sprint_planner", "tech_lead"):
        return fname, fname, 1

    for prefix in ("issue_writer", "workspace_setup", "workspace_cleanup", "integration_tester"):
        if fname.startswith(prefix + "_"):
            rest = fname[len(prefix) + 1 :]
            if "_iter_" in rest:
                parts = rest.rsplit("_iter_", 1)
                issue = parts[0]
                try:
                    iteration = int(parts[1])
                except ValueError:
                    iteration = 1
            else:
                issue = rest
                iteration = 1
            return prefix, issue, iteration

    parts = fname.split("_", 1)
    category = parts[0]
    rest = parts[1] if len(parts) > 1 else category
    if "_iter_" in rest:
        p = rest.rsplit("_iter_", 1)
        issue = p[0]
        try:
            iteration = int(p[1])
        except ValueError:
            iteration = 1
    else:
        issue = rest
        iteration = 1
    return category, issue, iteration


# ── helpers for aggregation ───────────────────────────────────────────
def _group_sum(rows: list[dict], key: str, val: str) -> dict[str, float]:
    acc: dict[str, float] = defaultdict(float)
    for r in rows:
        acc[r[key]] += r[val]
    return dict(acc)


def _group_count(rows: list[dict], key: str) -> dict[str, int]:
    acc: dict[str, int] = defaultdict(int)
    for r in rows:
        acc[r[key]] += 1
    return dict(acc)


def _pivot(rows: list[dict], index: str, column: str, value: str) -> tuple[list[str], list[str], list[list[float]]]:
    """Build a pivot table. Returns (row_labels, col_labels, values[][])."""
    grid: dict[str, dict[str, float]] = defaultdict(lambda: defaultdict(float))
    all_cols: set[str] = set()
    for r in rows:
        grid[r[index]][r[column]] += r[value]
        all_cols.add(r[column])
    row_labels = sorted(grid.keys())
    col_labels = [c for c in PHASE_ORDER if c in all_cols]  # ordered
    values = [[grid[row].get(col, 0.0) for col in col_labels] for row in row_labels]
    return row_labels, col_labels, values


# ── chart data preparation ────────────────────────────────────────────
def prepare_chart_data(rows: list[dict]) -> dict:
    t0 = min(r["ts_start"] for r in rows)
    t_end = max(r["ts_end"] for r in rows)
    total_wall_s = t_end - t0

    data: dict = {}
    data["meta"] = {"palette": CATEGORY_PALETTE, "phase_order": PHASE_ORDER}

    # 1. cost_treemap
    cost_by_cat = _group_sum(rows, "category", "cost_usd")
    data["cost_treemap"] = [{"category": k, "cost_usd": v} for k, v in cost_by_cat.items()]

    # 2. time_treemap
    dur_by_cat = _group_sum(rows, "category", "duration_s")
    data["time_treemap"] = [{"category": k, "duration_min": v / 60} for k, v in dur_by_cat.items()]

    # 3. burn_rate
    by_end = sorted(rows, key=lambda r: r["ts_end"])
    cum = 0.0
    burn = []
    for r in by_end:
        cum += r["cost_usd"]
        burn.append({"elapsed_min": (r["ts_end"] - t0) / 60, "cum_cost": cum})
    data["burn_rate"] = burn

    # 4. parallelism
    events: list[tuple[float, int]] = []
    for r in rows:
        events.append((r["ts_start"] - t0, 1))
        events.append((r["ts_end"] - t0, -1))
    events.sort(key=lambda x: (x[0], x[1]))
    par_data = []
    current = 0
    for t, d in events:
        current += d
        par_data.append({"time_min": t / 60, "concurrent": current})
    data["parallelism"] = par_data

    # 5. cost_efficiency
    count_by_cat = _group_count(rows, "category")
    eff = []
    for cat in cost_by_cat:
        total_min = dur_by_cat.get(cat, 0) / 60
        total_cost = cost_by_cat[cat]
        cpm = total_cost / max(total_min, 0.01)
        eff.append({"category": cat, "total_min": total_min, "total_cost": total_cost, "cost_per_min": cpm})
    data["cost_efficiency"] = eff

    # 6. time_heatmap
    issue_cats = {"coder", "reviewer", "qa", "synthesizer", "issue_writer"}
    sub = [r for r in rows if r["category"] in issue_cats]
    rl, cl, vals = _pivot(sub, "issue", "category", "duration_s")
    data["time_heatmap"] = {
        "issues": rl,
        "categories": cl,
        "values": [[v / 60 for v in row] for row in vals],
    }

    # 7. cost_heatmap
    rl_c, cl_c, vals_c = _pivot(sub, "issue", "category", "cost_usd")
    data["cost_heatmap"] = {"issues": rl_c, "categories": cl_c, "values": vals_c}

    # 8. duration_violin
    data["duration_violin"] = [{"category": r["category"], "duration_min": r["duration_s"] / 60} for r in rows]

    # 9. effort_scatter
    data["effort_scatter"] = [
        {"num_turns": r["num_turns"], "tool_calls": r["tool_calls"], "cost_usd": r["cost_usd"], "category": r["category"]}
        for r in rows
    ]

    # 10. pipeline_flow
    data["pipeline_flow"] = [
        {"category": r["category"], "start_min": (r["ts_start"] - t0) / 60, "dur_min": r["duration_s"] / 60, "issue": r["issue"]}
        for r in sorted(rows, key=lambda r: r["ts_start"])
    ]

    # 11. parallelism_ratio
    order = [c for c in PHASE_ORDER if c in dur_by_cat]
    data["parallelism_ratio"] = [{"category": c, "ratio": dur_by_cat[c] / total_wall_s} for c in order]

    # 12. issue_ranking
    rank_cats = {"coder", "reviewer", "qa", "synthesizer"}
    rank_sub = [r for r in rows if r["category"] in rank_cats]
    # pivot: issue → {phase: cost}
    issue_costs: dict[str, dict[str, float]] = defaultdict(lambda: defaultdict(float))
    for r in rank_sub:
        issue_costs[r["issue"]][r["category"]] += r["cost_usd"]
    # sort by total ascending
    sorted_issues = sorted(issue_costs.keys(), key=lambda iss: sum(issue_costs[iss].values()))
    rank_phases = [c for c in ["coder", "reviewer", "qa", "synthesizer"] if any(c in issue_costs[i] for i in issue_costs)]
    data["issue_ranking"] = [
        {"issue": iss, "phase_costs": {p: issue_costs[iss].get(p, 0.0) for p in rank_phases}}
        for iss in sorted_issues
    ]

    # 13. rework
    rework_cats = {"qa", "reviewer"}
    cost_ordered = [(c, cost_by_cat.get(c, 0.0)) for c in PHASE_ORDER if c in cost_by_cat]
    data["rework"] = [{"category": c, "cost_usd": v, "is_rework": c in rework_cats} for c, v in cost_ordered]

    # 14. dashboard KPIs
    total_cost = sum(r["cost_usd"] for r in rows)
    total_wall_min = total_wall_s / 60
    total_agent_min = sum(r["duration_s"] for r in rows) / 60
    total_turns = sum(r["num_turns"] for r in rows)
    total_tool_calls = sum(r["tool_calls"] for r in rows)
    num_agents = len(rows)
    peak = _peak_parallelism(rows)
    avg_cost = total_cost / num_agents if num_agents else 0

    data["dashboard"] = [
        {"label": "Total Cost", "value": f"${total_cost:.2f}", "color": "#e15759"},
        {"label": "Wall-Clock Time", "value": f"{total_wall_min:.1f} min", "color": "#4e79a7"},
        {"label": "Agent-Time", "value": f"{total_agent_min:.1f} min", "color": "#59a14f"},
        {"label": "Parallelism Factor", "value": f"{total_agent_min / total_wall_min:.1f}\u00d7", "color": "#f28e2b"},
        {"label": "Agent Runs", "value": f"{num_agents}", "color": "#76b7b2"},
        {"label": "Peak Concurrency", "value": f"{peak}", "color": "#b07aa1"},
        {"label": "Total Turns", "value": f"{total_turns:,}", "color": "#edc948"},
        {"label": "Total Tool Calls", "value": f"{total_tool_calls:,}", "color": "#9c755f"},
        {"label": "Avg Cost / Agent", "value": f"${avg_cost:.2f}", "color": "#ff9da7"},
    ]

    return data


def _peak_parallelism(rows: list[dict]) -> int:
    events: list[tuple[float, int]] = []
    for r in rows:
        events.append((r["ts_start"], 1))
        events.append((r["ts_end"], -1))
    events.sort()
    peak = current = 0
    for _, delta in events:
        current += delta
        peak = max(peak, current)
    return peak


# ── main ───────────────────────────────────────────────────────────────
def main() -> None:
    print("Parsing logs …")
    rows = parse_logs()
    categories = set(r["category"] for r in rows)
    total_cost = sum(r["cost_usd"] for r in rows)
    wall_min = (max(r["ts_end"] for r in rows) - min(r["ts_start"] for r in rows)) / 60
    print(f"  Found {len(rows)} agent executions across {len(categories)} categories")
    print(f"  Total cost: ${total_cost:.2f}")
    print(f"  Wall-clock: {wall_min:.1f} min\n")

    print("Preparing chart data …")
    chart_data = prepare_chart_data(rows)

    with tempfile.NamedTemporaryFile(mode="w", suffix=".json", delete=False) as f:
        json.dump(chart_data, f)
        json_path = f.name
    print(f"  Wrote chart data to {json_path}")

    renderer = SCRIPT_DIR / "render_charts.js"
    charts_dir = str(CHARTS_DIR)
    cmd = [
        "deno", "run",
        "--allow-read", "--allow-write", "--allow-env",
        str(renderer),
        json_path,
        charts_dir,
    ]
    print(f"\nRendering charts → {CHARTS_DIR}/")
    result = subprocess.run(cmd, capture_output=True, text=True)

    if result.stdout:
        print(result.stdout, end="")
    if result.stderr:
        print(result.stderr, end="", file=sys.stderr)

    if result.returncode != 0:
        print(f"\nRenderer failed with exit code {result.returncode}", file=sys.stderr)
        sys.exit(1)

    svg_files = sorted(CHARTS_DIR.glob("*.svg"))
    print(f"\n  Generated {len(svg_files)} SVGs")

    # Convert SVG → PNG using macOS sips
    print("\nConverting SVGs to PNGs …")
    png_count = 0
    for svg_path in svg_files:
        png_path = svg_path.with_suffix(".png")
        sips_cmd = ["sips", "-s", "format", "png", str(svg_path), "--out", str(png_path)]
        sips_result = subprocess.run(sips_cmd, capture_output=True, text=True)
        if sips_result.returncode == 0:
            png_count += 1
            print(f"  ✓ {png_path.name}")
        else:
            print(f"  ✗ {png_path.name}: {sips_result.stderr.strip()}", file=sys.stderr)

    print(f"\nDone — {png_count} PNGs in {CHARTS_DIR}")


if __name__ == "__main__":
    main()
