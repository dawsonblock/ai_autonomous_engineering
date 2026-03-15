from __future__ import annotations

from typing import Any, Dict, List

from aae.core.task_graph import ActionGraph


class PlanStep:
    __slots__ = ("step_id", "action", "description", "depends_on", "metadata")

    def __init__(
        self,
        step_id: str,
        action: str,
        description: str = "",
        depends_on: List[str] | None = None,
        metadata: Dict[str, Any] | None = None,
    ) -> None:
        self.step_id = step_id
        self.action = action
        self.description = description
        self.depends_on = depends_on or []
        self.metadata = metadata or {}


class LongHorizonPlanner:
    _TEMPLATES: Dict[str, List[Dict[str, Any]]] = {
        "fix_failing_test": [
            {"step_id": "analyze", "action": "analyze_code", "depends_on": []},
            {"step_id": "locate", "action": "identify_bug", "depends_on": ["analyze"]},
            {"step_id": "design", "action": "propose_patch", "depends_on": ["locate"]},
            {"step_id": "apply", "action": "apply_patch", "depends_on": ["design"]},
            {"step_id": "test", "action": "run_tests", "depends_on": ["apply"]},
            {"step_id": "evaluate", "action": "evaluate", "depends_on": ["test"]},
        ],
        "implement_feature": [
            {"step_id": "analyze", "action": "analyze_code", "depends_on": []},
            {"step_id": "design", "action": "design_solution", "depends_on": ["analyze"]},
            {"step_id": "implement", "action": "write_code", "depends_on": ["design"]},
            {"step_id": "test", "action": "run_tests", "depends_on": ["implement"]},
            {"step_id": "review", "action": "review_code", "depends_on": ["test"]},
            {"step_id": "evaluate", "action": "evaluate", "depends_on": ["review"]},
        ],
        "research": [
            {"step_id": "collect", "action": "collect_sources", "depends_on": []},
            {"step_id": "extract", "action": "extract_claims", "depends_on": ["collect"]},
            {"step_id": "hypothesize", "action": "generate_hypothesis", "depends_on": ["extract"]},
            {"step_id": "experiment", "action": "run_experiment", "depends_on": ["hypothesize"]},
            {"step_id": "evaluate", "action": "evaluate_results", "depends_on": ["experiment"]},
            {"step_id": "update", "action": "update_knowledge", "depends_on": ["evaluate"]},
        ],
    }

    def plan(self, goal: str, context: Dict[str, Any] | None = None) -> List[PlanStep]:
        template_key = self._match_template(goal)
        template = self._TEMPLATES.get(template_key, self._TEMPLATES["fix_failing_test"])
        return [
            PlanStep(
                step_id=entry["step_id"],
                action=entry["action"],
                depends_on=entry["depends_on"],
            )
            for entry in template
        ]

    def plan_as_graph(self, goal: str, context: Dict[str, Any] | None = None) -> ActionGraph:
        steps = self.plan(goal, context)
        graph = ActionGraph()
        for step in steps:
            graph.add_task(step.step_id, {"action": step.action})
        for step in steps:
            for dep in step.depends_on:
                graph.add_edge(dep, step.step_id)
        return graph

    def _match_template(self, goal: str) -> str:
        goal_lower = goal.lower()
        if any(keyword in goal_lower for keyword in ("test", "fix", "bug", "fail")):
            return "fix_failing_test"
        if any(keyword in goal_lower for keyword in ("feature", "implement", "add", "create")):
            return "implement_feature"
        if any(keyword in goal_lower for keyword in ("research", "paper", "experiment", "claim")):
            return "research"
        return "fix_failing_test"
