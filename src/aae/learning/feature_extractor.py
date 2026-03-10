from __future__ import annotations

from math import log
from typing import Dict, List


class FeatureExtractor:
    def extract(
        self,
        task_state: dict,
        graph_context: dict,
        branch_result: dict | None = None,
        recent_failures: List[str] | None = None,
    ) -> Dict[str, float]:
        recent_failures = recent_failures or []
        branch_result = branch_result or {}
        symbol_context = graph_context.get("symbol_context", [])
        covering_tests = graph_context.get("covering_tests", [])
        call_chains = graph_context.get("call_chains", [])
        changed_line_count = float(branch_result.get("patch_candidate", {}).get("changed_line_count", 0) or 0)
        risk_score = float(branch_result.get("patch_candidate", {}).get("simulation", {}).get("risk_score", 0.0))
        failure_entropy = 0.0
        if recent_failures:
            probability = 1.0 / len(recent_failures)
            failure_entropy = -len(recent_failures) * probability * log(probability)
        return {
            "repo_size": float(len(symbol_context) + len(call_chains)),
            "num_tests": float(len(covering_tests)),
            "graph_depth": float(max((chain.count("->") + 1 for chain in call_chains), default=0)),
            "failure_entropy": float(round(failure_entropy, 3)),
            "risk_score": risk_score,
            "changed_line_count": changed_line_count,
            "task_type_hash": float(len(str(task_state.get("task_type", "")))),
        }
