import json
from pathlib import Path

import pytest

from aae.agents.micro_agents.coding.patch_writer_agent import PatchWriterAgent
from aae.contracts.micro_agents import PatchPlan
from aae.evaluation.benchmark_runner import BenchmarkRunner
from aae.exploration.branch_generator import BranchGenerator
from aae.patching.patch_generator import HybridPatchGenerator


BENCHMARK_REPO = Path(__file__).resolve().parents[1] / "fixtures" / "benchmark_guard_repo"


@pytest.mark.anyio
async def test_patch_writer_generates_bounded_guard_clause():
    agent = PatchWriterAgent(patch_generator=HybridPatchGenerator(llm_provider=None))
    result = await agent.run(
        task={"goal": "Fix authenticate empty token failure"},
        context={
            "repo_path": str(BENCHMARK_REPO),
            "selected_plan": PatchPlan(
                id="plan_guard_inputs",
                summary="Add a guard clause around the primary failure path in authenticate",
                confidence=0.8,
                target_files=["auth.py"],
                strategy="input_validation",
                template_family="null_guard",
                declared_intents=[],
            ).model_dump(mode="json"),
            "semantic_context": {},
            "graph_context": {},
            "suspicious_locations": [
                {
                    "file_path": "auth.py",
                    "symbol": "authenticate",
                    "start_line": 4,
                    "end_line": 8,
                    "confidence": 0.91,
                    "evidence_sources": ["stack_trace"],
                    "score_components": {"stack_trace": 0.91},
                }
            ],
            "evidence": [{"source": "stack_trace", "file_path": "auth.py", "symbol": "authenticate", "line": 4, "weight": 0.91}],
        },
    )

    assert result["syntax_valid"] is True
    assert not result["validation_errors"]
    assert result["changed_files"] == ["auth.py"]
    assert "if not token:" in result["diff"]
    assert result["changed_line_count"] <= 20


@pytest.mark.anyio
async def test_benchmark_runner_produces_local_report(tmp_path: Path):
    corpus_path = tmp_path / "benchmark_corpus.json"
    corpus_path.write_text(
        json.dumps(
            [
                {
                    "case_id": "guard-empty-token",
                    "repo_path": str(BENCHMARK_REPO),
                    "goal": "Fix authenticate empty token failure",
                    "expected_file": "auth.py",
                    "expected_function": "authenticate",
                    "expected_edit_lines": [4, 8],
                }
            ]
        ),
        encoding="utf-8",
    )
    runner = BenchmarkRunner(corpus_path=str(corpus_path), artifacts_dir=str(tmp_path / "evaluation"))

    report = await runner.run()

    assert report["records"]
    assert report["records"][0]["baseline_returncode"] != 0
    assert report["records"][0]["selected_branch_id"]
    assert report["records"][0]["branch_succeeded"] is True
    assert report["records"][0]["trust_level"] in {"strict", "degraded"}
    assert report["records"][0]["fixed"] is (report["records"][0]["trust_level"] == "strict")
    assert "strict_fix_rate" in report["metrics"]
    assert report["records"][0]["localization_metrics"]["file_top1"] is True
    assert "metrics" in report
    assert Path(report["report_path"]).exists()
    assert Path(report["markdown_report_path"]).exists()


def test_branch_generator_avoids_duplicate_branch_ids():
    generator = BranchGenerator()
    branches = generator.generate(
        planner_decision={
            "selected_branch_id": "branch_2",
            "branches": [
                {"branch_id": "branch_1", "metadata": {}},
                {"branch_id": "branch_2", "metadata": {}},
                {"branch_id": "branch_3", "metadata": {}},
            ],
        },
        swarm_result={
            "consensus_decision": {"selected_plan_id": "plan_b"},
            "test_impact": {"tests": ["tests/test_auth.py"]},
            "patch_candidates": [
                {"plan_id": "plan_a", "diff": "--- a/auth.py\n+++ b/auth.py\n", "simulation": {"test_prediction": {"affected_tests": [".sandbox_artifacts/local-sandbox-1/workspace/tests/test_auth.py"]}}},
                {"plan_id": "plan_b", "diff": "--- a/auth.py\n+++ b/auth.py\n", "simulation": {"test_prediction": {"affected_tests": []}}},
                {"plan_id": "plan_c", "diff": "--- a/auth.py\n+++ b/auth.py\n", "simulation": {"test_prediction": {"affected_tests": []}}},
            ],
        },
    )

    branch_ids = [branch["branch_id"] for branch in branches]
    assert len(branch_ids) == len(set(branch_ids))
    assert branches[0]["selected_tests"] == ["tests/test_auth.py"]
