from __future__ import annotations

import json
from pathlib import Path
from typing import Any, Dict

from aae.behavior_model.behavior_query_engine import BehaviorQueryEngine
from aae.behavior_model.state_graph_builder import StateGraphBuilder
from aae.behavior_model.state_transition_store import StateTransitionStore
from aae.contracts.behavior import TraceRecord
from aae.code_analysis.call_signature_resolver import CallSignatureResolver
from aae.code_analysis.context_ranker import ContextRanker
from aae.code_analysis.cfg_builder import CfgBuilder
from aae.code_analysis.type_inference import TypeInferenceEngine
from aae.contracts.tasks import TaskSpec
from aae.events.event_bus import EventBus
from aae.graph.graph_query import GraphQueryEngine
from aae.graph.repo_graph_builder import RepoGraphBuilder
from aae.memory.base import MemoryStore
from aae.persistence.graph_store import PostgresGraphStore
from aae.persistence.trajectory_store import PostgresTrajectoryStore
from aae.runtime.workspace import RepoMaterializer
from aae.tools.graph_tools import GraphContextBuilder


class RuntimeTaskPreparer:
    def __init__(
        self,
        memory: MemoryStore,
        event_bus: EventBus,
        artifacts_dir: str = ".artifacts",
        swe_preparation_service: "SWEPreparationService | None" = None,
        materializer: RepoMaterializer | None = None,
    ) -> None:
        self.memory = memory
        self.event_bus = event_bus
        self.artifacts_dir = artifacts_dir
        self.materializer = materializer or RepoMaterializer(artifacts_dir=artifacts_dir)
        self.swe_preparation_service = swe_preparation_service or SWEPreparationService(
            memory=memory,
            event_bus=event_bus,
            artifacts_dir=artifacts_dir,
            materializer=self.materializer,
        )

    async def prepare(self, workflow_id: str, task: TaskSpec, memory_snapshot: Dict[str, Any]) -> TaskSpec:
        if "repo_url" in task.payload or "repo_path" in task.payload:
            task = await self._ensure_workspace(workflow_id, task, memory_snapshot)
        if task.task_type == "swe_build":
            task = await self.swe_preparation_service.prepare(workflow_id, task, memory_snapshot)
        return task

    async def _ensure_workspace(self, workflow_id: str, task: TaskSpec, memory_snapshot: Dict[str, Any]) -> TaskSpec:
        workflow_ns = "workflow/%s" % workflow_id
        workspace_data = memory_snapshot.get("repo_workspace") or self.memory.get(workflow_ns, "repo_workspace")
        if workspace_data is None:
            workspace = await self.materializer.materialize(
                workflow_id=workflow_id,
                repo_url=task.payload.get("repo_url"),
                repo_path=task.payload.get("repo_path"),
            )
            workspace_data = workspace.model_dump(mode="json")
            self.memory.put(workflow_ns, "repo_workspace", workspace_data)
        payload = dict(task.payload)
        payload["repo_path"] = workspace_data["repo_path"]
        return task.model_copy(update={"payload": payload})


class SWEPreparationService:
    def __init__(
        self,
        memory: MemoryStore,
        event_bus: EventBus,
        artifacts_dir: str = ".artifacts",
        materializer: RepoMaterializer | None = None,
    ) -> None:
        from aae.agents.micro_agents.orchestration.swarm_controller import SwarmController
        from aae.exploration.branch_generator import BranchGenerator
        from aae.exploration.experiment_runner import ExperimentRunner
        from aae.exploration.result_comparator import ResultComparator
        from aae.learning.tool_router import ToolRouter
        from aae.memory.graph_memory import GraphMemory
        from aae.memory.trajectory_memory import TrajectoryMemory
        from aae.planner.planner_runtime import PlannerRuntime

        self.memory = memory
        self.event_bus = event_bus
        self.artifacts_dir = artifacts_dir
        self.materializer = materializer or RepoMaterializer(artifacts_dir=artifacts_dir)
        self.graph_builder = RepoGraphBuilder()
        self.graph_memory = GraphMemory(base_dir=str(Path(artifacts_dir) / "memory" / "graphs"))
        self.trajectory_memory = TrajectoryMemory(base_dir=str(Path(artifacts_dir) / "memory" / "trajectories"))
        self.behavior_builder = StateGraphBuilder()
        self.behavior_store = StateTransitionStore(base_dir=str(Path(artifacts_dir) / "memory" / "behavior"))
        self.cfg_builder = CfgBuilder()
        self.type_inference = TypeInferenceEngine()
        self.signature_resolver = CallSignatureResolver()
        self.context_ranker = ContextRanker()
        self.tool_router = ToolRouter()
        self.swarm = SwarmController()
        self.planner_runtime = PlannerRuntime()
        self.branch_generator = BranchGenerator()
        self.experiment_runner = ExperimentRunner()
        self.result_comparator = ResultComparator()
        self.persistent_graph_store = PostgresGraphStore()
        self.persistent_trajectory_store = PostgresTrajectoryStore()

    async def prepare(self, workflow_id: str, task: TaskSpec, memory_snapshot: Dict[str, Any]) -> TaskSpec:
        workflow_ns = "workflow/%s" % workflow_id
        workspace_data = memory_snapshot.get("repo_workspace") or self.memory.get(workflow_ns, "repo_workspace")
        if workspace_data is None:
            workspace = await self.materializer.materialize(
                workflow_id=workflow_id,
                repo_url=task.payload.get("repo_url"),
                repo_path=task.payload.get("repo_path"),
            )
            workspace_data = workspace.model_dump(mode="json")
            self.memory.put(workflow_ns, "repo_workspace", workspace_data)

        repo_path = workspace_data["repo_path"]
        graph_build = self.memory.get(workflow_ns, "graph_build")
        if graph_build is None:
            graph_dir = Path(self.artifacts_dir) / "graphs" / workflow_id
            graph_dir.mkdir(parents=True, exist_ok=True)
            build_result = self.graph_builder.build(
                repo_path=repo_path,
                sqlite_path=str(graph_dir / "repo_graph.sqlite3"),
                json_path=str(graph_dir / "repo_graph.json"),
            )
            graph_build = build_result.model_dump(mode="json")
            self.memory.put(workflow_ns, "graph_build", graph_build)
            self.graph_memory.store(workflow_id, build_result)
            self.persistent_graph_store.store_build_result(workflow_id, build_result)

        graph = GraphQueryEngine.from_sqlite(graph_build["sqlite_path"])
        goal = str(task.payload.get("goal", ""))
        behavior_model = self.memory.get(workflow_ns, "behavior_model")
        if behavior_model is None:
            behavior_snapshot = self.behavior_builder.build(repo_path=repo_path, graph_snapshot=graph.snapshot)
            behavior_model = self.behavior_store.store_snapshot(workflow_id, behavior_snapshot)
            self.memory.put(workflow_ns, "behavior_model", behavior_model)
        
        thread_id = str(task.payload.get("thread_id", workflow_id))
        self.persistent_trajectory_store.save_checkpoint(
            namespace="swe_preparation",
            thread_id=thread_id,
            state={"status": "prepared", "workspace": workspace_data, "graph_build": graph_build},
        )
        behavior_snapshot = self.behavior_store.load_snapshot(workflow_id)
        behavior_engine = BehaviorQueryEngine(behavior_snapshot) if behavior_snapshot is not None else None
        trace_records = [trace.model_dump(mode="json") for trace in self.behavior_store.load_traces(workflow_id)]
        preliminary_graph_context = GraphContextBuilder(graph, context_ranker=self.context_ranker).build(goal)
        behavior_context = self._build_behavior_context(behavior_engine, preliminary_graph_context) if behavior_engine is not None else {}
        graph_context = GraphContextBuilder(graph, context_ranker=self.context_ranker).build(goal, behavior_context=behavior_context, failure_evidence=trace_records)
        self.memory.put(workflow_ns, "trace_artifacts", trace_records)
        self.memory.put(workflow_ns, "behavior_context", behavior_context)
        self.memory.put(workflow_ns, "ranked_context", {key: graph_context.get(key, []) for key in ["ranked_symbols", "ranked_files", "ranked_snippets"]})
        semantic_context = self._build_semantic_context(repo_path, graph, graph_context)
        self.memory.put(workflow_ns, "semantic_context", semantic_context)
        tool_recommendations = self.tool_router.route(
            task_state={"task_type": task.task_type, "goal": goal},
            graph_context=graph_context,
            prior_actions=memory_snapshot.get("planner_actions", []),
            recent_failures=[result.get("error", {}).get("message", "") for result in (memory_snapshot.get("task_results") or {}).values() if result.get("error")],
        )
        swarm_result = await self.swarm.run(
            task=task.payload,
            context={
                "repo_path": repo_path,
                "graph": graph,
                "behavior_engine": behavior_engine,
                "graph_context": graph_context,
                "behavior_context": behavior_context,
                "semantic_context": semantic_context,
                "trace_records": trace_records,
                "memory_state": memory_snapshot,
                "recent_failures": [result.get("error", {}).get("message", "") for result in (memory_snapshot.get("task_results") or {}).values() if result.get("error")],
                "sandbox_runs": self.memory.get(workflow_ns, "sandbox_runs") or [],
                **graph_context,
            },
        )
        self.memory.put(workflow_ns, "bug_localization", swarm_result.get("bug_localization", {}))
        self.memory.put(workflow_ns, "patch_candidates", swarm_result.get("patch_candidates", []))
        self.memory.put(
            workflow_ns,
            "simulation_results",
            [candidate.get("simulation", {}) for candidate in swarm_result.get("patch_candidates", []) if candidate.get("simulation")],
        )
        planner_decision = self.planner_runtime.plan(
            workflow_goal=goal,
            graph_context=graph_context,
            memory_state=memory_snapshot,
            swarm_result=swarm_result,
        )
        exploration_branches = self.branch_generator.generate(planner_decision.model_dump(mode="json"), swarm_result)
        exploration_results = await self.experiment_runner.run(
            repo_path=repo_path,
            branches=exploration_branches,
            artifacts_dir=str(Path(self.artifacts_dir) / "sandbox" / workflow_id),
        )
        appended_traces = []
        for result in exploration_results:
            for trace_path in result.get("execution", {}).get("metadata", {}).get("trace_paths", []):
                for trace in self._load_trace_records(trace_path):
                    appended_traces.append(trace)
        if appended_traces:
            self.behavior_store.append_traces(workflow_id, appended_traces)
        branch_comparison = self.result_comparator.compare(exploration_results).model_dump(mode="json")
        self.memory.put(workflow_ns, "sandbox_runs", [result.get("execution", {}).get("metadata", {}) for result in exploration_results])
        self.memory.put(workflow_ns, "exploration_results", exploration_results)
        self.memory.put(workflow_ns, "branch_comparison", branch_comparison)
        self.memory.put(workflow_ns, "evaluation_runs", [{"workflow_id": workflow_id, "selected_branch_id": branch_comparison.get("selected_branch_id", ""), "result_count": branch_comparison.get("summary", {}).get("result_count", 0)}])
        branch_memory_records = [record.model_dump(mode="json") for record in self.planner_runtime.planner.branch_memory.records()]
        execution_by_branch = {result.get("branch_id", ""): result.get("execution", {}).get("metadata", {}) for result in exploration_results}
        for record in branch_memory_records:
            execution_metadata = execution_by_branch.get(record.get("branch_id", ""), {})
            if execution_metadata:
                record["metadata"] = {
                    **record.get("metadata", {}),
                    "patch_apply_status": execution_metadata.get("patch_apply_status", ""),
                    "rollback_status": execution_metadata.get("rollback_status", ""),
                    "repair_loop": execution_metadata.get("repair_loop", {}),
                    "counterexample_paths": execution_metadata.get("counterexample_paths", []),
                }
        self.memory.put(workflow_ns, "branch_memory", branch_memory_records)
        
        self.persistent_trajectory_store.save_checkpoint(
            namespace="swe_preparation",
            thread_id=thread_id + "/exploration",
            state={
                "status": "explored",
                "planner_decision": planner_decision.model_dump(mode="json"),
                "branch_comparison": branch_comparison,
            },
            parent_thread_id=thread_id,
        )

        trajectory_record = {
            "workflow_id": workflow_id,
            "task_id": task.task_id,
            "repo_path": repo_path,
            "graph_context": graph_context,
            "behavior_context": behavior_context,
            "semantic_context": semantic_context,
            "tool_recommendations": tool_recommendations,
            "planner_decision": planner_decision.model_dump(mode="json"),
            "exploration_results": exploration_results,
            "branch_comparison": branch_comparison,
        }
        self.trajectory_memory.append("swe_preparation", trajectory_record)
        self.persistent_trajectory_store.append("swe_preparation", trajectory_record)

        payload = dict(task.payload)
        payload.update(
            {
                "repo_path": repo_path,
                "repo_workspace": workspace_data,
                "graph_build": graph_build,
                "graph_context": graph_context,
                "behavior_model": behavior_model,
                "behavior_context": behavior_context,
                "semantic_context": semantic_context,
                "tool_recommendations": tool_recommendations,
                "swarm_context": swarm_result,
                "planner_decision": planner_decision.model_dump(mode="json"),
                "exploration_results": exploration_results,
                "branch_comparison": branch_comparison,
            }
        )
        self.memory.put(workflow_ns, "planner_actions", [action.model_dump(mode="json") for action in planner_decision.branches[0].actions] if planner_decision.branches else [])
        self.memory.put(workflow_ns, "swe_preparation", trajectory_record)
        return task.model_copy(update={"payload": payload})

    def _build_semantic_context(self, repo_path: str, graph: GraphQueryEngine, graph_context: Dict[str, Any]) -> Dict[str, Any]:
        semantic_context: Dict[str, Any] = {}
        for entry in graph_context.get("symbol_context", []):
            for match in entry.get("matches", [])[:2]:
                summary = self.cfg_builder.build_for_symbol(
                    repo_path=repo_path,
                    file_path=match["path"],
                    symbol_id=match["id"],
                    qualname=match["qualname"],
                )
                inferred_types = self.type_inference.infer_for_function(
                    repo_path=repo_path,
                    file_path=match["path"],
                    function_name=match["name"],
                )
                resolved = self.signature_resolver.resolve(graph.snapshot, match["qualname"])
                semantic_context[match["name"]] = {
                    "cfg_nodes": summary.cfg_nodes,
                    "branch_points": summary.branch_points,
                    "inferred_types": inferred_types,
                    "signature": resolved.get("signature", ""),
                    "resolved_calls": resolved.get("resolved_calls", []),
                }
        return semantic_context

    def _build_behavior_context(self, behavior_engine: BehaviorQueryEngine, graph_context: Dict[str, Any]) -> Dict[str, Any]:
        candidate_symbols = graph_context.get("candidate_symbols", [])
        suspicious_files = behavior_engine.suspicious_files(candidate_symbols).items
        causal_paths = []
        for symbol in candidate_symbols[:3]:
            causal_paths.extend(item["path"] for item in behavior_engine.causal_path(symbol).items[:3])
        return {
            "suspicious_files": suspicious_files,
            "causal_paths": causal_paths[:9],
            "trace_overlap": behavior_engine.trace_overlap(candidate_symbols).items[:8],
        }

    def _load_trace_records(self, trace_path: str) -> list[TraceRecord]:
        path = Path(trace_path)
        if not path.exists():
            return []
        records = []
        for line in path.read_text(encoding="utf-8").splitlines():
            if not line.strip():
                continue
            try:
                records.append(TraceRecord.model_validate(json.loads(line)))
            except json.JSONDecodeError:
                continue
        return records
