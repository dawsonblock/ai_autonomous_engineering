from __future__ import annotations

from aae.learning.dataset_builder import DatasetBuilder
from aae.learning.tool_policy_model import ToolPolicyModel
from aae.learning.trajectory_parser import TrajectoryParser


class PolicyTrainer:
    def __init__(
        self,
        parser: TrajectoryParser | None = None,
        dataset_builder: DatasetBuilder | None = None,
        model: ToolPolicyModel | None = None,
    ) -> None:
        self.parser = parser or TrajectoryParser()
        self.dataset_builder = dataset_builder or DatasetBuilder()
        self.model = model or ToolPolicyModel()

    def train_from_paths(self, paths: list[str]) -> ToolPolicyModel:
        trajectories = self.parser.parse_many(paths)
        dataset = self.dataset_builder.build(trajectories)
        return self.model.fit(dataset)
