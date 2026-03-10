from __future__ import annotations

import json
from pathlib import Path
from typing import Any, Dict

from aae.code_analysis.call_signature_resolver import CallSignatureResolver
from aae.code_analysis.cfg_builder import CfgBuilder
from aae.code_analysis.type_inference import TypeInferenceEngine
from aae.contracts.tasks import TaskSpec
from aae.events.event_bus import EventBus
from aae.graph.graph_query import GraphQueryEngine
from aae.graph.repo_graph_builder import RepoGraphBuilder
from aae.memory.base import MemoryStore
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
        self.cfg_builder = CfgBuilder()
        self.type_inference = TypeInferenceEngine()
        self.signature_resolver = CallSignatureResolver()
        self.tool_router = ToolRouter()
        self.swarm = SwarmController()
        self.planner_runtime = PlannerRuntime()

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

        graph = GraphQueryEngine.from_sqlite(graph_build["sqlite_path"])
        goal = str(task.payload.get("goal", ""))
        graph_context = GraphContextBuilder(graph).build(goal)
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
                "graph_context": graph_context,
                "semantic_context": semantic_context,
                **graph_context,
            },
        )
        planner_decision = self.planner_runtime.plan(
            workflow_goal=goal,
            graph_context=graph_context,
            memory_state=memory_snapshot,
            swarm_result=swarm_result,
        )

        trajectory_record = {
            "workflow_id": workflow_id,
            "task_id": task.task_id,
            "repo_path": repo_path,
            "graph_context": graph_context,
            "semantic_context": semantic_context,
            "tool_recommendations": tool_recommendations,
            "planner_decision": planner_decision.model_dump(mode="json"),
        }
        self.trajectory_memory.append("swe_preparation", trajectory_record)

        payload = dict(task.payload)
        payload.update(
            {
                "repo_path": repo_path,
                "repo_workspace": workspace_data,
                "graph_build": graph_build,
                "graph_context": graph_context,
                "semantic_context": semantic_context,
                "tool_recommendations": tool_recommendations,
                "swarm_context": swarm_result,
                "planner_decision": planner_decision.model_dump(mode="json"),
            }
        )
        self.memory.put(workflow_ns, "planner_actions", [action.model_dump(mode="json") for action in planner_decision.branches[0].actions] if planner_decision.branches else [])
        self.memory.put(workflow_ns, "branch_memory", planner_decision.rationale.get("branch_memory", []))
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
