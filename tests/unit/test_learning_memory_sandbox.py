import json
import sys
from pathlib import Path

import pytest

from aae.contracts.graph import GraphBuildResult, GraphSnapshot
from aae.learning.dataset_builder import DatasetBuilder
from aae.learning.policy_trainer import PolicyTrainer
from aae.learning.tool_router import ToolRouter
from aae.learning.trajectory_parser import TrajectoryParser
from aae.learning.trajectory_stats import TrajectoryStats
from aae.memory.graph_memory import GraphMemory
from aae.memory.trajectory_memory import TrajectoryMemory
from aae.memory.vector_memory import VectorMemory
from aae.sandbox.sandbox_api import SandboxAPI


def test_learning_pipeline_builds_dataset_and_tool_distribution(tmp_path: Path):
    log_path = tmp_path / "events.jsonl"
    records = [
        {"event_type": "task.succeeded", "workflow_id": "wf1", "payload": {"task_type": "swe_build", "tool": "graph_query"}},
        {"event_type": "task.failed", "workflow_id": "wf1", "payload": {"task_type": "swe_build", "tool": "open_file"}},
    ]
    log_path.write_text("\n".join(json.dumps(record) for record in records), encoding="utf-8")

    parser = TrajectoryParser()
    trajectories = parser.parse_jsonl(log_path)
    stats = TrajectoryStats().summarize(trajectories)
    dataset = DatasetBuilder().build(trajectories)
    model = PolicyTrainer().train_from_paths([str(log_path)])
    routing = ToolRouter(model=model).route(
        task_state={"task_type": "swe_build"},
        graph_context={"symbol_context": [{"matches": [1]}], "covering_tests": ["tests/test_auth.py"]},
        prior_actions=[],
        recent_failures=[],
    )

    assert stats["workflow_count"] == 1
    assert dataset[0]["tool"] == "graph_query"
    assert "graph_query" in routing


def test_persistent_memories_round_trip(tmp_path: Path):
    graph_memory = GraphMemory(base_dir=str(tmp_path / "graphs"))
    trajectory_memory = TrajectoryMemory(base_dir=str(tmp_path / "trajectories"))
    vector_memory = VectorMemory(path=str(tmp_path / "vectors.json"))

    build = GraphBuildResult(
        snapshot=GraphSnapshot(root_path="/tmp/repo", nodes=[], edges=[]),
        root_path="/tmp/repo",
        sqlite_path="/tmp/repo.sqlite3",
        json_path="/tmp/repo.json",
        stats={"file_count": 0},
    )
    graph_memory.store("wf_mem", build)
    trajectory_memory.append("runs", {"workflow_id": "wf_mem", "status": "ok"})
    vector_memory.put("repo", [1.0, 0.0], {"kind": "graph"})

    assert graph_memory.load("wf_mem")["root_path"] == "/tmp/repo"
    assert trajectory_memory.read("runs")[0]["status"] == "ok"
    assert vector_memory.search([1.0, 0.0])[0]["key"] == "repo"


@pytest.mark.anyio
async def test_sandbox_api_runs_commands(tmp_path: Path):
    script_path = tmp_path / "script.py"
    script_path.write_text("print('ok')\n", encoding="utf-8")
    sandbox = SandboxAPI()
    results = await sandbox.run_tests(str(tmp_path), ['%s %s' % (sys.executable, script_path.name)])

    assert results[0]["returncode"] == 0
    assert "ok" in results[0]["stdout"]
    assert results[0]["trace_paths"]
    assert results[0]["artifact_paths"]
    assert not (tmp_path / ".sandbox_artifacts").exists()
