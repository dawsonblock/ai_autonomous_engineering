from __future__ import annotations


class DatasetBuilder:
    def build(self, trajectories: list[dict]) -> list[dict]:
        rows = []
        for record in trajectories:
            payload = record.get("payload", {})
            repair_guidance = payload.get("repair_guidance", {})
            patch_apply = payload.get("patch_apply_details", {})
            branch_comparison = payload.get("branch_comparison", {})
            rows.append(
                {
                    "task_type": payload.get("task_type") or record.get("event_type", "unknown"),
                    "tool": payload.get("tool") or payload.get("strategy") or "graph_query",
                    "strategy": payload.get("strategy") or payload.get("template_family") or "",
                    "success": 1 if record.get("event_type") in {"task.succeeded", "benchmark.case_succeeded"} or payload.get("success") else 0,
                    "workflow_id": record.get("workflow_id", ""),
                    "case_id": payload.get("case_id", ""),
                    "graph_context": payload.get("graph_context", {}),
                    "runtime_cost_s": float(payload.get("runtime_cost_s", 0.0)),
                    "regression_count": int(payload.get("regression_count", 0) or 0),
                    "risk_score": float(payload.get("risk_score", 0.0)),
                    "repair_constraints": repair_guidance.get("constraints", []),
                    "repair_template": repair_guidance.get("preferred_template", ""),
                    "counterexample_count": int(payload.get("counterexample_count", len(payload.get("counterexamples", [])) or 0)),
                    "patch_apply_status": payload.get("patch_apply_status", ""),
                    "patch_apply_changed_files": patch_apply.get("results", [{}])[0].get("changed_files", []) if patch_apply.get("results") else [],
                    "reference_count": int(payload.get("reference_count", 0) or 0),
                    "selected_branch_id": branch_comparison.get("selected_branch_id", ""),
                    "benchmark_score": float(payload.get("benchmark_score", 0.0) or 0.0),
                }
            )
        return rows
