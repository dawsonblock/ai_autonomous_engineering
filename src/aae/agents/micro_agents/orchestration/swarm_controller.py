from __future__ import annotations

from aae.agents.micro_agents.coding.bug_localizer_agent import BugLocalizerAgent
from aae.agents.micro_agents.coding.dependency_tracer_agent import DependencyTracerAgent
from aae.agents.micro_agents.coding.dependency_risk_agent import DependencyRiskAgent
from aae.agents.micro_agents.coding.failure_analyzer_agent import FailureAnalyzerAgent
from aae.agents.micro_agents.coding.patch_planner_agent import PatchPlannerAgent
from aae.agents.micro_agents.coding.patch_reviewer_agent import PatchReviewerAgent
from aae.agents.micro_agents.coding.patch_writer_agent import PatchWriterAgent
from aae.agents.micro_agents.coding.regression_guard_agent import RegressionGuardAgent
from aae.agents.micro_agents.coding.repo_mapper_agent import RepoMapperAgent
from aae.agents.micro_agents.coding.symbol_locator_agent import SymbolLocatorAgent
from aae.agents.micro_agents.coding.test_impact_agent import TestImpactAgent
from aae.agents.micro_agents.orchestration.candidate_pool import CandidatePool
from aae.agents.micro_agents.orchestration.consensus_engine import ConsensusEngine
from aae.agents.micro_agents.orchestration.solution_judge import SolutionJudge
from aae.agents.micro_agents.registry import MicroAgentRegistry
from aae.contracts.planner import CandidatePlan
from aae.planner.simulation.patch_simulator import PatchSimulator


class SwarmController:
    def __init__(
        self,
        registry: MicroAgentRegistry | None = None,
        consensus_engine: ConsensusEngine | None = None,
        solution_judge: SolutionJudge | None = None,
        patch_simulator: PatchSimulator | None = None,
    ) -> None:
        self.registry = registry or MicroAgentRegistry()
        if not list(self.registry.list()):
            self.registry.register(RepoMapperAgent())
            self.registry.register(SymbolLocatorAgent())
            self.registry.register(DependencyTracerAgent())
            self.registry.register(BugLocalizerAgent())
            self.registry.register(TestImpactAgent())
            self.registry.register(DependencyRiskAgent())
            self.registry.register(PatchPlannerAgent())
            self.registry.register(PatchWriterAgent())
            self.registry.register(RegressionGuardAgent())
            self.registry.register(FailureAnalyzerAgent())
            self.registry.register(PatchReviewerAgent())
        self.consensus_engine = consensus_engine or ConsensusEngine()
        self.solution_judge = solution_judge or SolutionJudge()
        self.patch_simulator = patch_simulator or PatchSimulator()

    async def run(self, task, context):
        repo_map = await self.registry.get("repo_mapper").run(task, context)
        symbols = await self.registry.get("symbol_locator").run(task, {**context, **repo_map})
        dependencies = await self.registry.get("dependency_tracer").run(task, {**context, **repo_map, **symbols})
        bug_localization = await self.registry.get("bug_localizer").run(task, {**context, **repo_map, **symbols, **dependencies})
        test_impact = await self.registry.get("test_impact").run(task, {**context, **repo_map, **symbols, **dependencies, **bug_localization})
        dependency_risk = await self.registry.get("dependency_risk").run(task, {**context, **repo_map, **symbols, **dependencies, **bug_localization})
        plans = await self.registry.get("patch_planner").run(
            task,
            {**context, **repo_map, **symbols, **dependencies, **bug_localization, **test_impact, **dependency_risk},
        )

        pool = CandidatePool()
        for plan in plans.get("plans", []):
            pool.add(
                CandidatePlan(
                    plan_id=plan["id"],
                    summary=plan["summary"],
                    confidence=float(plan.get("confidence", 0.0)),
                    agent_name="patch_planner",
                    changed_files=plan.get("target_files", []),
                    impact_size=len(plan.get("target_files", [])),
                    risk_score=float(dependency_risk.get("risk_score", 0.0)),
                    risk_reasons=list(dependency_risk.get("risk_reasons", [])),
                    predicted_test_count=len(test_impact.get("tests", [])),
                )
            )
        shortlisted = self.consensus_engine.filter_candidates(pool.get_all())
        decision = self.solution_judge.select_best(shortlisted)
        patch_candidates = []
        for candidate in shortlisted[:3]:
            selected_plan = next(
                (plan for plan in plans.get("plans", []) if plan["id"] == candidate.plan_id),
                {},
            )
            patch = await self.registry.get("patch_writer").run(
                task,
                {
                    **context,
                    **repo_map,
                    **symbols,
                    **dependencies,
                    **bug_localization,
                    **test_impact,
                    **dependency_risk,
                    "selected_plan": selected_plan,
                },
            )
            simulation = self.patch_simulator.simulate(
                candidate_plan_id=patch.get("plan_id", ""),
                changed_files=patch.get("changed_files", []),
                graph=context["graph"],
                behavior=context.get("behavior_engine"),
            )
            patch_candidates.append(
                {
                    **patch,
                    "simulation": simulation.model_dump(mode="json"),
                    "plan_summary": selected_plan.get("summary", ""),
                }
            )
        selected_plan = next(
            (plan for plan in plans.get("plans", []) if plan["id"] == decision.selected_plan_id),
            plans.get("plans", [{}])[0] if plans.get("plans") else {},
        )
        patch = next(
            (candidate for candidate in patch_candidates if candidate.get("plan_id") == decision.selected_plan_id),
            patch_candidates[0] if patch_candidates else {},
        )
        simulation = patch.get("simulation")
        if simulation is None:
            simulation_model = self.patch_simulator.simulate(
                candidate_plan_id=patch.get("plan_id", ""),
                changed_files=patch.get("changed_files", []),
                graph=context["graph"],
                behavior=context.get("behavior_engine"),
            )
            simulation = simulation_model.model_dump(mode="json")
        else:
            simulation_model = None
        regression_guard = await self.registry.get("regression_guard").run(
            task,
            {**context, **patch, **dependency_risk, **simulation},
        )
        review = await self.registry.get("patch_reviewer").run(
            task,
            {**context, **patch, "simulation": simulation},
        )
        return {
            "repo_map": repo_map,
            "symbols": symbols,
            "dependencies": dependencies,
            "bug_localization": bug_localization,
            "test_impact": test_impact,
            "dependency_risk": dependency_risk,
            "plans": plans,
            "shortlisted_candidates": [candidate.model_dump(mode="json") for candidate in shortlisted],
            "consensus_decision": decision.model_dump(mode="json"),
            "selected_plan": selected_plan,
            "patch_candidate": patch,
            "patch_candidates": patch_candidates,
            "simulation": simulation,
            "regression_guard": regression_guard,
            "review": review,
        }
