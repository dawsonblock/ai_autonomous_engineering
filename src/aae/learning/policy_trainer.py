from __future__ import annotations

from aae.learning.dataset_builder import DatasetBuilder
from aae.learning.feature_extractor import FeatureExtractor
from aae.learning.policy_network import PolicyNetwork
from aae.learning.reward_model import RewardModel
from aae.learning.trajectory_parser import TrajectoryParser


class PolicyTrainer:
    def __init__(
        self,
        parser: TrajectoryParser | None = None,
        dataset_builder: DatasetBuilder | None = None,
        feature_extractor: FeatureExtractor | None = None,
        reward_model: RewardModel | None = None,
        model: PolicyNetwork | None = None,
    ) -> None:
        self.parser = parser or TrajectoryParser()
        self.dataset_builder = dataset_builder or DatasetBuilder()
        self.feature_extractor = feature_extractor or FeatureExtractor()
        self.reward_model = reward_model or RewardModel()
        self.model = model or PolicyNetwork()

    def train_from_paths(self, paths: list[str]) -> PolicyNetwork:
        trajectories = self.parser.parse_many(paths)
        dataset = self.dataset_builder.build(trajectories)
        enriched = []
        for row in dataset:
            features = self.feature_extractor.extract({"task_type": row.get("task_type", "")}, row.get("graph_context", {}), branch_result=row)
            reward = self.reward_model.score(row)
            enriched.append(
                {
                    **row,
                    **features,
                    "reward": reward,
                    "repair_template": row.get("repair_template", ""),
                    "counterexample_count": row.get("counterexample_count", 0),
                    "patch_apply_status": row.get("patch_apply_status", ""),
                }
            )
        return self.model.fit(enriched)
