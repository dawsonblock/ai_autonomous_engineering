from __future__ import annotations

from aae.learning.feature_extractor import FeatureExtractor
from aae.learning.policy_network import PolicyNetwork
from aae.learning.tool_policy_model import ToolPolicyModel


class ToolRouter:
    def __init__(
        self,
        model: ToolPolicyModel | PolicyNetwork | None = None,
        feature_extractor: FeatureExtractor | None = None,
    ) -> None:
        self.model = model
        self.feature_extractor = feature_extractor or FeatureExtractor()

    def route(
        self,
        task_state: dict,
        graph_context: dict,
        prior_actions: list[dict],
        recent_failures: list[str],
    ) -> dict[str, float]:
        ranking = {
            "graph_query": 0.5,
            "repo_search": 0.2,
            "open_file": 0.2,
            "test_selector": 0.1,
        }
        if graph_context.get("symbol_context"):
            ranking["graph_query"] += 0.2
        if graph_context.get("covering_tests"):
            ranking["test_selector"] += 0.2
        if recent_failures:
            ranking["open_file"] += 0.15
        if any(action.get("action_type") == "graph_query" for action in prior_actions):
            ranking["repo_search"] += 0.05
        if self.model is not None:
            learned = {}
            if hasattr(self.model, "predict_ranked"):
                features = self.feature_extractor.extract(task_state, graph_context, recent_failures=recent_failures)
                learned = self.model.predict_ranked(str(task_state.get("task_type", "")))
                if features.get("num_tests", 0) > 0:
                    learned["test_selector"] = learned.get("test_selector", 0.0) + 0.05
            else:
                learned = self.model.predict_proba(str(task_state.get("task_type", "")))
            for tool, probability in learned.items():
                ranking[tool] = ranking.get(tool, 0.0) * 0.5 + probability * 0.5
        total = sum(ranking.values())
        return dict(sorted(((tool, weight / total) for tool, weight in ranking.items()), key=lambda item: item[1], reverse=True))
