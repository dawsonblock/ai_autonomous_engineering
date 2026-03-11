from __future__ import annotations

import json
import sys
import uuid
from pathlib import Path
from typing import Any

from aae.contracts.tasks import TaskSpec
from aae.evaluation.localization_metrics import LocalizationMetrics
from aae.evaluation.metrics_reporter import MetricsReporter
from aae.evaluation.regression_tests import RegressionTests
from aae.evaluation.report_generator import ReportGenerator
from aae.events.event_bus import EventBus
from aae.events.event_logger import EventLogger
from aae.localization.serializers import LocalizationSerializer
from aae.memory.in_memory import InMemoryMemoryStore
from aae.meta.self_improvement_loop import SelfImprovementLoop
from aae.persistence.evaluation_store import PostgresEvaluationStore
from aae.runtime.swe_preparation import RuntimeTaskPreparer
from aae.sandbox.sandbox_api import SandboxAPI


class BenchmarkRunner:
    def __init__(
        self,
        corpus_path: str | None = None,
        artifacts_dir: str = ".artifacts/evaluation",
        sandbox_api: SandboxAPI | None = None,
    ) -> None:
        self.project_root = Path(__file__).resolve().parents[3]
        self.corpus_path = Path(corpus_path) if corpus_path else self.project_root / "evaluation" / "benchmark_corpus.json"
        self.artifacts_dir = Path(artifacts_dir)
        self.sandbox_api = sandbox_api or SandboxAPI()
        self.reporter = MetricsReporter()
        self.regression_tests = RegressionTests()
        self.report_generator = ReportGenerator()
        self.evaluation_store = PostgresEvaluationStore()
        self.self_improvement_loop = SelfImprovementLoop()

    def load_corpus(self) -> list[dict[str, Any]]:
        return json.loads(self.corpus_path.read_text(encoding="utf-8"))

    async def run(self) -> dict[str, Any]:
        run_id = "benchmark-%s" % uuid.uuid4().hex[:8]
        records = []
        for case in self.load_corpus():
            record = await self.run_case(case)
            records.append(record)
            self.evaluation_store.record_case(run_id, str(record.get("case_id", "")), record)
        metrics = self.reporter.summarize(records)
        regression_summary = self.regression_tests.evaluate(metrics)
        meta_result = self.self_improvement_loop.run(
            artifacts_dir=str(self.artifacts_dir),
            evaluation_records=records,
            patch_provenance=[provenance for record in records for provenance in record.get("patch_provenance", [])],
            trajectories=[
                {
                    "tool": record.get("selected_branch_id", ""),
                    "fixed": record.get("fixed", False),
                    "execution_mode": record.get("execution_mode", ""),
                }
                for record in records
            ],
        )
        report = {
            "run_id": run_id,
            "corpus_path": str(self.corpus_path),
            "records": records,
            "metrics": metrics,
            "regression_summary": regression_summary,
            "meta_profile": meta_result,
        }
        report_path = self.reporter.write_report(self.artifacts_dir / "benchmark_report.json", report)
        markdown_path = self.report_generator.write_markdown(self.artifacts_dir / "benchmark_report.md", report)
        self.evaluation_store.record_summary(run_id, report)
        return {**report, "report_path": report_path, "markdown_report_path": markdown_path}

    async def run_case(self, case: dict[str, Any]) -> dict[str, Any]:
        case_id = str(case.get("case_id") or "case-%s" % uuid.uuid4().hex[:8])
        repo_path = self._resolve_repo_path(str(case["repo_path"]))
        goal = str(case["goal"])
        test_command = str(case.get("test_command") or ("%s -m pytest -q" % sys.executable))

        baseline_results = await self.sandbox_api.run_tests(str(repo_path), [test_command])
        baseline = baseline_results[0]

        memory = InMemoryMemoryStore()
        event_bus = EventBus(logger=EventLogger(artifacts_dir=str(self.artifacts_dir / case_id / "events")))
        preparer = RuntimeTaskPreparer(memory=memory, event_bus=event_bus, artifacts_dir=str(self.artifacts_dir / case_id))
        workflow_id = "benchmark_%s" % case_id
        task = TaskSpec(
            task_id="swe_build",
            task_type="swe_build",
            agent_name="swe_af",
            payload={"goal": goal, "repo_path": str(repo_path)},
        )
        prepared = await preparer.prepare(workflow_id, task, {})
        workflow_ns = "workflow/%s" % workflow_id
        exploration_results = memory.get(workflow_ns, "exploration_results") or prepared.payload.get("exploration_results", [])
        branch_comparison = memory.get(workflow_ns, "branch_comparison") or prepared.payload.get("branch_comparison", {})
        selected_branch_id = branch_comparison.get("selected_branch_id", "")
        selected_result = next((result for result in exploration_results if result.get("branch_id") == selected_branch_id), exploration_results[0] if exploration_results else {})
        execution = selected_result.get("execution", {})
        execution_metadata = execution.get("metadata", {})
        execution_returncode = execution_metadata.get("returncode")
        if execution_returncode is None:
            execution_returncode = execution_metadata.get("exit_code", 1)
        execution_returncode = int(execution_returncode)
        execution_mode = str(execution_metadata.get("execution_mode") or ("docker" if execution_metadata.get("transport") == "docker" else "local"))
        trust_level = str(execution_metadata.get("trust_level") or ("strict" if execution_mode == "docker" else "degraded"))
        branch_succeeded = baseline.get("returncode", 0) != 0 and execution_returncode == 0
        fixed = branch_succeeded and trust_level == "strict"
        patch_candidate = selected_result.get("patch_candidate", {}) or prepared.payload.get("swarm_context", {}).get("patch_candidate", {})
        selected_tests = list(selected_result.get("selected_tests", []))

        localization_result = memory.get(workflow_ns, "localization_ranked_context") or memory.get(workflow_ns, "bug_localization") or {}
        localization_metrics = self._localization_metrics(case, localization_result)

        evaluation_record = (memory.get(workflow_ns, "evaluation_runs") or [{}])[0]
        evaluation_record = {
            **evaluation_record,
            "execution_mode": execution_mode,
            "trust_level": trust_level,
            "branch_succeeded": branch_succeeded,
        }

        return {
            "case_id": case_id,
            "goal": goal,
            "repo_path": str(repo_path),
            "baseline_returncode": int(baseline.get("returncode", 0) or 0),
            "baseline_execution_mode": baseline.get("execution_mode", ""),
            "baseline_trust_level": baseline.get("trust_level", ""),
            "selected_branch_id": selected_branch_id,
            "fixed": fixed,
            "branch_succeeded": branch_succeeded,
            "execution_mode": execution_mode,
            "trust_level": trust_level,
            "fallback_reason": execution_metadata.get("fallback_reason", ""),
            "patch_size": int(patch_candidate.get("changed_line_count", 0) or 0),
            "runtime_cost_s": float(execution.get("runtime_cost_s", 0.0) or 0.0),
            "regression_count": int(execution.get("regression_count", 0) or 0),
            "selected_test_count": len(selected_tests),
            "suspicious_locations": localization_result,
            "localization_metrics": localization_metrics,
            "evaluation_record": evaluation_record,
            "patch_provenance": memory.get(workflow_ns, "patch_provenance") or [],
        }

    def _resolve_repo_path(self, repo_path: str) -> Path:
        path = Path(repo_path)
        return path if path.is_absolute() else self.project_root / path

    def _localization_metrics(self, case: dict[str, Any], localization_payload: dict[str, Any]) -> dict[str, Any]:
        expected_file = str(case.get("expected_file") or "")
        if not expected_file:
            return {}
        result = LocalizationSerializer.from_agent_payload(localization_payload)
        expected_function = case.get("expected_function")
        expected_edit_lines = case.get("expected_edit_lines")
        true_span = tuple(expected_edit_lines) if isinstance(expected_edit_lines, list) and len(expected_edit_lines) == 2 else None
        return LocalizationMetrics.calculate(result, true_file=expected_file, true_function=expected_function, true_span=true_span)
