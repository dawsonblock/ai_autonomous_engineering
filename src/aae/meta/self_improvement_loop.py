from __future__ import annotations

import json
from pathlib import Path
from typing import Any

from aae.meta.experiment_manager import ExperimentManager
from aae.meta.strategy_optimizer import StrategyOptimizer
from aae.meta.tool_evaluator import ToolEvaluator


class SelfImprovementLoop:
    def __init__(
        self,
        strategy_optimizer: StrategyOptimizer | None = None,
        tool_evaluator: ToolEvaluator | None = None,
        experiment_manager: ExperimentManager | None = None,
    ) -> None:
        self.strategy_optimizer = strategy_optimizer or StrategyOptimizer()
        self.tool_evaluator = tool_evaluator or ToolEvaluator()
        self.experiment_manager = experiment_manager or ExperimentManager()

    def run(self, artifacts_dir: str, evaluation_records: list[dict[str, Any]], patch_provenance: list[dict[str, Any]], trajectories: list[dict[str, Any]]) -> dict[str, Any]:
        strategy_profile = self.strategy_optimizer.rank(evaluation_records, patch_provenance)
        tool_profile = self.tool_evaluator.summarize(trajectories)
        profile = self.experiment_manager.propose(strategy_profile, tool_profile)
        profile_path = Path(artifacts_dir) / "meta" / "strategy_profile.json"
        profile_path.parent.mkdir(parents=True, exist_ok=True)
        profile_path.write_text(json.dumps(profile, indent=2, sort_keys=True), encoding="utf-8")
        return {"profile": profile, "profile_path": str(profile_path)}
