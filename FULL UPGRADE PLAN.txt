Below is a structured engineering prompt plan to implement the upgrades discussed. It is designed so you can feed the prompts to an LLM-based coding agent step-by-step to evolve the current system. The plan assumes the existing repositories (af-deep-research, sec-af, SWE-AF) remain the base.

The plan is divided into implementation phases. Each phase contains LLM engineering prompts intended to produce code modules.

⸻

Phase 1 — System Integration Layer (AI Kernel)

Goal: unify the three repositories under one controller.

Prompt 1 — Controller Architecture

Design a modular AI controller service that orchestrates multiple agent subsystems.

Existing subsystems:
- Research agent (af-deep-research)
- Security analysis agent (sec-af)
- Autonomous software engineering agent (SWE-AF)

Requirements:
- Python implementation
- asynchronous execution (asyncio)
- agent registry
- task scheduler
- dependency graph execution
- retry logic for failed tasks
- structured event system
- memory interface

Controller loop:

while tasks exist:
    select ready task
    dispatch to appropriate agent
    collect result
    update system memory
    emit events

Deliver:
1. controller.py
2. task_scheduler.py
3. agent_registry.py
4. event_bus.py
5. task_graph.py


⸻

Phase 2 — Event Bus (Agent Communication)

Goal: decouple agents using an event-driven system.

Prompt 2 — Event System

Implement an event-driven communication system for a multi-agent platform.

Requirements:
- publish/subscribe architecture
- asynchronous event handling
- Redis or in-memory fallback
- typed event schema
- event logging
- event replay capability

Example events:
- research_completed
- vulnerability_detected
- patch_generated
- test_failed

Deliver:
1. event_bus.py
2. event_types.py
3. event_listener.py
4. event_logger.py


⸻

Phase 3 — Repository Graph (Environment Model)

Goal: build a persistent structural representation of code.

Prompt 3 — Code Graph Builder

Create a repository graph builder for analyzing software projects.

Capabilities:
- parse source code using tree-sitter
- extract files, classes, functions
- build call graph
- build dependency graph
- connect tests to functions

Graph schema:

Nodes:
- File
- Class
- Function
- Module
- Test

Edges:
- CALLS
- IMPORTS
- DEFINES
- TESTS

Storage:
- Neo4j graph database
- fallback: NetworkX in-memory graph

Deliver modules:
1. repo_graph_builder.py
2. ast_parser.py
3. dependency_extractor.py
4. graph_store.py
5. graph_query_api.py


⸻

Phase 4 — Graph Query Interface

Goal: allow agents to reason using the repository graph.

Prompt 4 — Graph Tooling

Build a query interface allowing agents to retrieve structured code information.

Supported queries:
- locate functions related to a symbol
- retrieve call chain
- find files affecting a test
- identify modules importing a dependency

Interface example:

graph.find_call_chain("authenticate")
graph.tests_covering_function("parse_token")

Deliver:
1. graph_query_tool.py
2. graph_agent_interface.py
3. graph_context_builder.py


⸻

Phase 5 — Planning Engine (Search-Based Reasoning)

Goal: replace simple reactive loops with action planning.

Prompt 5 — Planning System

Design a planning engine for autonomous coding agents.

Capabilities:
- action tree search
- candidate patch generation
- plan scoring
- beam search pruning
- branch simulation

Planning cycle:

state → generate candidate actions → score branches → execute best branch → update plan

Deliver modules:
1. planner.py
2. action_tree.py
3. beam_search.py
4. plan_evaluator.py


⸻

Phase 6 — Simulation Layer

Goal: evaluate candidate solutions before execution.

Prompt 6 — Outcome Simulation

Implement a simulation system that predicts effects of proposed code changes.

Inputs:
- candidate patch
- repository graph

Simulation outputs:
- affected functions
- dependency ripple effects
- likely test failures

Deliver modules:
1. patch_simulator.py
2. dependency_impact.py
3. predicted_failure_model.py


⸻

Phase 7 — Trajectory Learning

Goal: learn from existing JSONL logs in SWE-AF.

Prompt 7 — Trajectory Analyzer

Build a trajectory analysis module for agent execution logs.

Input:
- JSONL files from SWE-AF runs

Extract:
- task success rates
- tool usage patterns
- patch attempt counts
- token costs

Output:
- policy training dataset

Deliver modules:
1. trajectory_parser.py
2. trajectory_stats.py
3. tool_policy_dataset.py


⸻

Phase 8 — Policy Model for Tool Selection

Goal: improve tool choice using learned behavior.

Prompt 8 — Tool Policy Model

Create a machine learning model that predicts optimal tool selection.

Inputs:
- task state
- repository context
- previous actions

Outputs:
- probability distribution over tools

Model:
- lightweight transformer or gradient boosted trees

Deliver modules:
1. tool_policy_model.py
2. tool_selector.py
3. policy_training_pipeline.py


⸻

Phase 9 — Memory System

Goal: persistent system knowledge.

Prompt 9 — Memory Architecture

Implement a multi-layer memory system.

Layers:

Short-term memory
- current task state

Long-term memory
- knowledge graph
- vulnerability database

Trajectory memory
- agent execution history

Deliver modules:
1. memory_manager.py
2. vector_store.py
3. graph_memory.py
4. trajectory_store.py


⸻

Phase 10 — Sandbox Execution Cluster

Goal: scalable test execution.

Prompt 10 — Sandbox System

Design a distributed sandbox execution environment.

Features:
- Docker container pool
- job scheduling
- secure isolation
- test execution API

Deliver modules:
1. sandbox_manager.py
2. container_pool.py
3. job_executor.py


⸻

Phase 11 — Agent Upgrade Prompts

Upgrade each subsystem to use the new infrastructure.

Research Agent Prompt

Modify the research agent so it:

- stores extracted knowledge in the system memory graph
- emits research_completed events
- supports structured knowledge queries


⸻

Security Agent Prompt

Upgrade the security agent so it:

- uses repository graph to identify attack surfaces
- builds attack graphs
- emits vulnerability_detected events


⸻

SWE Agent Prompt

Upgrade the SWE agent so it:

- queries the repository graph for code navigation
- uses the planning engine for patch generation
- runs simulation before applying patches
- logs trajectories for learning


⸻

Phase 12 — Unified Runtime

Prompt 12 — Full Runtime

Combine all components into a unified runtime.

Runtime responsibilities:
- start controller
- initialize agents
- connect event bus
- connect memory systems
- launch sandbox workers

Deliver:
1. runtime.py
2. config.yaml
3. system_launcher.py


⸻

Phase 13 — Monitoring and Metrics

Prompt 13 — Observability

Implement monitoring for the AI engineering platform.

Track:
- agent success rates
- task duration
- token costs
- tool usage

Provide:
- Prometheus metrics
- simple dashboard

Deliver:
1. metrics_collector.py
2. dashboard_server.py


⸻

Final Architecture After Upgrades

User Request
      ↓
Controller (AI Kernel)
      ↓
Planning Engine
      ↓
Agent Pool
  ├ Research
  ├ Security
  └ SWE
      ↓
Graph + Memory System
      ↓
Tool Layer
      ↓
Sandbox Cluster


⸻

If you want, I can also generate something extremely useful for this project:

A complete repository structure for the upgraded system (roughly 200+ modules) so the entire platform can be built systematically instead of evolving chaotically.

Below is a clean repository structure for the upgraded platform. It organizes the controller, agents, planning system, graph layer, memory, and sandbox infrastructure into a single coherent codebase. The structure is designed so the system can scale without becoming unmanageable.

⸻

Top-Level Repository Layout

ai_autonomous_engineering/
│
├── controller/
├── agents/
├── planner/
├── graph/
├── memory/
├── tools/
├── sandbox/
├── learning/
├── monitoring/
├── runtime/
├── configs/
├── datasets/
└── tests/

Each directory corresponds to a subsystem.

⸻

1. Controller (AI Kernel)

The controller orchestrates all agents and system components.

controller/
│
├── controller.py
├── scheduler.py
├── task_graph.py
├── agent_registry.py
├── dependency_manager.py
├── retry_manager.py
└── event_bus.py

Responsibilities
	•	schedule tasks
	•	manage agent lifecycle
	•	dispatch jobs
	•	coordinate system state
	•	emit and listen for events

⸻

2. Agents

Agents implement domain reasoning.

agents/
│
├── research_agent/
│   ├── research_agent.py
│   ├── search_pipeline.py
│   ├── document_parser.py
│   └── synthesis_engine.py
│
├── security_agent/
│   ├── security_agent.py
│   ├── attack_surface_mapper.py
│   ├── exploit_generator.py
│   └── vulnerability_detector.py
│
├── swe_agent/
│   ├── swe_agent.py
│   ├── repo_explorer.py
│   ├── patch_generator.py
│   └── test_runner.py
│
└── base_agent/
    ├── base_agent.py
    └── agent_interface.py

Each agent inherits from a shared interface.

⸻

3. Planning System

Handles action tree search and solution evaluation.

planner/
│
├── planner.py
├── action_tree.py
├── beam_search.py
├── plan_evaluator.py
├── simulation_engine.py
└── patch_planner.py

Key tasks
	•	generate candidate solutions
	•	score branches
	•	prune low-probability paths

⸻

4. Repository Graph

Builds and stores the structural model of the codebase.

graph/
│
├── repo_graph_builder.py
├── ast_parser.py
├── dependency_extractor.py
├── call_graph_builder.py
├── graph_store.py
└── graph_query.py

Graph contents

Nodes:
	•	file
	•	class
	•	function
	•	module
	•	test

Edges:
	•	calls
	•	imports
	•	defines
	•	tests

⸻

5. Memory System

Persistent knowledge across runs.

memory/
│
├── memory_manager.py
├── short_term_memory.py
├── vector_memory.py
├── graph_memory.py
├── trajectory_store.py
└── knowledge_store.py

Memory types

Short-term:
	•	task state
	•	intermediate results

Long-term:
	•	knowledge graph
	•	vulnerability database

Trajectory:
	•	agent execution logs

⸻

6. Tools

Agents interact with the environment through tools.

tools/
│
├── filesystem_tool.py
├── repo_search_tool.py
├── shell_tool.py
├── browser_tool.py
├── graph_query_tool.py
└── tool_registry.py

Tools provide standardized interfaces.

Example:

tool.execute(input_data)


⸻

7. Sandbox Execution

Handles safe execution of external actions.

sandbox/
│
├── sandbox_manager.py
├── container_pool.py
├── job_executor.py
├── docker_runner.py
└── sandbox_api.py

Capabilities:
	•	run tests
	•	execute exploits
	•	compile code

All operations occur in isolated containers.

⸻

8. Learning System

Uses trajectory logs to improve decision-making.

learning/
│
├── trajectory_parser.py
├── trajectory_stats.py
├── policy_dataset_builder.py
├── tool_policy_model.py
└── policy_trainer.py

The SWE JSONL logs become training data.

⸻

9. Monitoring and Metrics

Tracks system performance.

monitoring/
│
├── metrics_collector.py
├── dashboard_server.py
├── task_monitor.py
└── logging_config.py

Metrics include:
	•	task success rate
	•	token usage
	•	agent runtime

⸻

10. Runtime Layer

Launches the platform.

runtime/
│
├── runtime.py
├── system_launcher.py
├── agent_loader.py
└── service_registry.py

The runtime initializes:
	•	controller
	•	agents
	•	memory
	•	event system

⸻

11. Configuration

configs/
│
├── system_config.yaml
├── agent_config.yaml
├── planner_config.yaml
└── sandbox_config.yaml

Configuration controls:
	•	model endpoints
	•	sandbox limits
	•	agent settings

⸻

12. Datasets

datasets/
│
├── trajectory_logs/
├── vulnerability_data/
└── research_corpus/

These datasets feed the learning modules.

⸻

13. Testing

tests/
│
├── controller_tests/
├── agent_tests/
├── graph_tests/
├── planner_tests/
└── integration_tests/

Tests verify each subsystem.

⸻

Final Architecture

The finished system runs as:

User Request
      │
      ▼
Controller
      │
      ▼
Planning Engine
      │
      ▼
Agent Pool
      │
      ▼
Graph + Memory
      │
      ▼
Tool Layer
      │
      ▼
Sandbox Execution


⸻

Next Logical Step

The structure above enables a stable system, but there is one more upgrade that significantly increases capability:

introducing a multi-agent reasoning swarm where specialized micro-agents collaborate on each task instead of a single reasoning chain.

That change dramatically improves success rates for complex engineering problems.

You create them as small, single-purpose workers inside the agents/ layer, then register them with the controller like any other agent. Do not start with dozens. Start with a micro-agent pool for one domain, make the handoffs explicit, then expand.

Where they live

In the repo structure I gave, add a new branch under agents/:

agents/
├── base_agent/
├── research_agent/
├── security_agent/
├── swe_agent/
└── micro_agents/
    ├── __init__.py
    ├── registry.py
    ├── schemas.py
    ├── orchestration/
    │   ├── swarm_controller.py
    │   ├── handoff_router.py
    │   └── consensus.py
    ├── coding/
    │   ├── repo_mapper_agent.py
    │   ├── symbol_locator_agent.py
    │   ├── dependency_tracer_agent.py
    │   ├── patch_planner_agent.py
    │   ├── patch_writer_agent.py
    │   ├── test_selector_agent.py
    │   ├── failure_analyzer_agent.py
    │   └── patch_reviewer_agent.py
    ├── security/
    │   ├── attack_surface_agent.py
    │   ├── input_flow_agent.py
    │   ├── exploit_hypothesis_agent.py
    │   └── vuln_verifier_agent.py
    └── research/
        ├── source_finder_agent.py
        ├── evidence_grader_agent.py
        ├── contradiction_checker_agent.py
        └── synthesis_agent.py

That is the cleanest place because micro-agents are still agents. They are just narrower and cheaper.

How to think about them

A micro-agent should do one job well and return a structured result. It should not try to solve the whole task.

Bad micro-agent:
	•	“fix bug agent”

Good micro-agents:
	•	“find files related to failing test”
	•	“trace call chain from symbol X”
	•	“generate three candidate minimal patches”
	•	“explain why patch failed after test run”

The rule is simple:

If the result can be represented as a small typed object, it can be a micro-agent.

How they fit into the system

Do not let the user-facing controller call ten micro-agents directly. Put a swarm controller between the main task and the micro-agents.

Flow:

Main Controller
   ↓
Domain Agent (SWE / Security / Research)
   ↓
Swarm Controller
   ↓
Micro-agents
   ↓
Structured outputs
   ↓
Aggregator / consensus
   ↓
Domain Agent result

So for a coding task:

SWE Agent
  ├─ Repo Mapper
  ├─ Symbol Locator
  ├─ Dependency Tracer
  ├─ Patch Planner
  ├─ Patch Writer
  ├─ Test Selector
  ├─ Failure Analyzer
  └─ Patch Reviewer

The first micro-agents to build

For your system, start with the SWE side. It gives the fastest gain.

Build these first:

1. repo_mapper_agent.py

Purpose: summarize repo structure relevant to the task.

Input:

{
    "task_id": "123",
    "issue_text": "...",
    "repo_path": "/workspace/repo"
}

Output:

{
    "candidate_files": ["a.py", "b.py"],
    "entrypoints": ["main.py:run", "api.py:create_app"],
    "tests": ["tests/test_auth.py"],
    "confidence": 0.81
}

2. symbol_locator_agent.py

Purpose: find symbols tied to the bug or feature.

Output:

{
    "symbols": [
        {"name": "authenticate", "file": "auth.py", "line": 88},
        {"name": "parse_token", "file": "jwt.py", "line": 41}
    ]
}

3. dependency_tracer_agent.py

Purpose: trace imports, calls, and impacted areas.

Output:

{
    "call_chain": [
        "api.login -> auth.authenticate -> jwt.parse_token"
    ],
    "impacted_tests": ["tests/test_login.py"]
}

4. patch_planner_agent.py

Purpose: propose several minimal fix strategies.

Output:

{
    "plans": [
        {"id": "plan_a", "summary": "validate empty token before decode"},
        {"id": "plan_b", "summary": "normalize bearer prefix in parser"}
    ]
}

5. patch_writer_agent.py

Purpose: write a patch for one selected plan.

Output:

{
    "plan_id": "plan_a",
    "diff": "...unified diff...",
    "changed_files": ["jwt.py"]
}

6. test_selector_agent.py

Purpose: choose the smallest useful test set.

Output:

{
    "commands": [
        "pytest tests/test_login.py -q",
        "pytest tests/test_auth.py -q"
    ]
}

7. failure_analyzer_agent.py

Purpose: read test failure output and explain the root cause.

Output:

{
    "failure_type": "logic_error",
    "suspected_file": "jwt.py",
    "reason": "empty token path still reaches decode()"
}

8. patch_reviewer_agent.py

Purpose: review the patch before execution.

Output:

{
    "accept": true,
    "risks": ["may not cover prefixed token format"],
    "followups": ["add regression test for empty string input"]
}

The base interface

All micro-agents should share one interface. Keep it strict.

from abc import ABC, abstractmethod
from typing import Any, Dict

class BaseMicroAgent(ABC):
    name: str
    domain: str

    @abstractmethod
    async def run(self, task: Dict[str, Any], context: Dict[str, Any]) -> Dict[str, Any]:
        """Execute one narrow task and return structured output."""
        raise NotImplementedError

Then each agent implements run() and returns JSON-serializable output only. No freeform chaos.

The registry

You need a registry so the swarm controller can discover and invoke them.

class MicroAgentRegistry:
    def __init__(self):
        self._agents = {}

    def register(self, agent):
        self._agents[agent.name] = agent

    def get(self, name: str):
        return self._agents[name]

    def list_by_domain(self, domain: str):
        return [a for a in self._agents.values() if a.domain == domain]

The swarm controller

This is where the orchestration happens. Keep the first version simple.

class SwarmController:
    def __init__(self, registry):
        self.registry = registry

    async def run_swe_swarm(self, task, context):
        repo_map = await self.registry.get("repo_mapper").run(task, context)
        symbols = await self.registry.get("symbol_locator").run(task, {**context, **repo_map})
        deps = await self.registry.get("dependency_tracer").run(task, {**context, **repo_map, **symbols})
        plans = await self.registry.get("patch_planner").run(task, {**context, **repo_map, **symbols, **deps})
        return {
            "repo_map": repo_map,
            "symbols": symbols,
            "dependencies": deps,
            "plans": plans,
        }

Then the parent SWE agent chooses a plan, calls patch_writer, runs tests, and sends failures back through failure_analyzer.

How they communicate

Do not let them pass giant prompts to each other. Pass small structured objects through shared schemas.

Create agents/micro_agents/schemas.py with Pydantic models like:

from pydantic import BaseModel
from typing import List, Optional

class CandidateFile(BaseModel):
    path: str
    reason: str
    score: float

class RepoMapResult(BaseModel):
    candidate_files: List[CandidateFile]
    tests: List[str]
    confidence: float

This matters. Without schemas the swarm becomes prompt spaghetti.

How many you should run at once

At first, keep it narrow:
	•	3 to 5 micro-agents per task
	•	one orchestrator
	•	one reviewer

Too many micro-agents creates overhead and disagreement. Most systems get worse before they get better when they over-fragment.

The practical pattern that works

Use this split:

Phase A: map
	•	repo mapper
	•	symbol locator
	•	dependency tracer

Phase B: plan
	•	patch planner
	•	test selector

Phase C: execute
	•	patch writer
	•	sandbox test run

Phase D: critique
	•	failure analyzer
	•	patch reviewer

That gives you a stable loop without agent explosion.

Where the actual “creation” happens

You create them in three places at once:

In code

Create the Python module under agents/micro_agents/...

In config

Add entries in configs/agent_config.yaml:

micro_agents:
  repo_mapper:
    enabled: true
    domain: swe
    model: fast-model
  symbol_locator:
    enabled: true
    domain: swe
    model: fast-model
  patch_planner:
    enabled: true
    domain: swe
    model: strong-model

In orchestration

Register them inside runtime startup:

registry.register(RepoMapperAgent())
registry.register(SymbolLocatorAgent())
registry.register(DependencyTracerAgent())
registry.register(PatchPlannerAgent())

Which model to use for each

Do not waste your strongest model everywhere.

Use:
	•	cheap/fast model for map, locate, classify, select
	•	stronger model for patch planning, patch writing, review
	•	non-LLM tools where possible for repo graph, grep, AST, tests

That keeps cost under control.

What not to do

Do not make micro-agents that:
	•	have overlapping responsibilities
	•	rewrite each other’s outputs
	•	depend on large hidden prompts
	•	return vague natural language instead of typed objects
	•	run without a shared state contract

That is how swarm systems collapse into noise.

The best first implementation

If you want the fastest path, implement this exact minimum set first:

agents/micro_agents/coding/
  repo_mapper_agent.py
  symbol_locator_agent.py
  dependency_tracer_agent.py
  patch_planner_agent.py
  patch_writer_agent.py
  failure_analyzer_agent.py
  patch_reviewer_agent.py

agents/micro_agents/
  registry.py
  schemas.py

agents/micro_agents/orchestration/
  swarm_controller.py

That is enough to convert a single-chain SWE agent into a real micro-agent system.

The next step after that

Once this works, add:
	•	a consensus layer where two planners propose plans and one reviewer scores them
	•	a policy router that decides which micro-agents to invoke for each task type
	•	a trajectory logger per micro-agent so you can learn which combinations actually help

If you want, I can generate the actual scaffold for this micro-agent subsystem as code files, starting with the base classes, schemas, registry, and first 6 SWE micro-agents.

Below is a minimal but complete scaffold for the micro-agent subsystem. It includes base classes, schemas, registry, a swarm controller, and the first set of SWE micro-agents. The code is intentionally small and deterministic so you can extend it safely.

⸻

Directory Layout

agents/micro_agents/
│
├── base.py
├── registry.py
├── schemas.py
│
├── orchestration/
│   └── swarm_controller.py
│
└── coding/
    ├── repo_mapper_agent.py
    ├── symbol_locator_agent.py
    ├── dependency_tracer_agent.py
    ├── patch_planner_agent.py
    ├── patch_writer_agent.py
    ├── failure_analyzer_agent.py
    └── patch_reviewer_agent.py


⸻

1. Base Micro-Agent Interface

base.py

from abc import ABC, abstractmethod
from typing import Dict, Any


class BaseMicroAgent(ABC):
    name: str
    domain: str

    def __init__(self):
        if not hasattr(self, "name"):
            raise ValueError("Agent must define name")
        if not hasattr(self, "domain"):
            raise ValueError("Agent must define domain")

    @abstractmethod
    async def run(self, task: Dict[str, Any], context: Dict[str, Any]) -> Dict[str, Any]:
        """
        Execute one focused operation and return structured output.
        """
        pass


⸻

2. Micro-Agent Registry

registry.py

class MicroAgentRegistry:
    def __init__(self):
        self._agents = {}

    def register(self, agent):
        self._agents[agent.name] = agent

    def get(self, name):
        return self._agents.get(name)

    def list(self):
        return list(self._agents.keys())

    def list_by_domain(self, domain):
        return [a for a in self._agents.values() if a.domain == domain]


⸻

3. Shared Schemas

schemas.py

from pydantic import BaseModel
from typing import List


class CandidateFile(BaseModel):
    path: str
    reason: str
    score: float


class RepoMapResult(BaseModel):
    candidate_files: List[CandidateFile]
    tests: List[str]
    confidence: float


class PatchPlan(BaseModel):
    id: str
    summary: str


class PatchResult(BaseModel):
    plan_id: str
    diff: str
    changed_files: List[str]


⸻

4. Swarm Controller

orchestration/swarm_controller.py

class SwarmController:

    def __init__(self, registry):
        self.registry = registry

    async def run_swe_swarm(self, task, context):

        repo_mapper = self.registry.get("repo_mapper")
        symbol_locator = self.registry.get("symbol_locator")
        dependency_tracer = self.registry.get("dependency_tracer")
        patch_planner = self.registry.get("patch_planner")

        repo_map = await repo_mapper.run(task, context)

        symbols = await symbol_locator.run(
            task, {**context, **repo_map}
        )

        dependencies = await dependency_tracer.run(
            task, {**context, **symbols}
        )

        plans = await patch_planner.run(
            task, {**context, **symbols, **dependencies}
        )

        return {
            "repo_map": repo_map,
            "symbols": symbols,
            "dependencies": dependencies,
            "plans": plans
        }


⸻

5. Repo Mapper Micro-Agent

coding/repo_mapper_agent.py

import os
from agents.micro_agents.base import BaseMicroAgent


class RepoMapperAgent(BaseMicroAgent):

    name = "repo_mapper"
    domain = "swe"

    async def run(self, task, context):

        repo_path = task.get("repo_path")

        files = []
        for root, _, filenames in os.walk(repo_path):
            for f in filenames:
                if f.endswith(".py"):
                    files.append(os.path.join(root, f))

        candidate_files = files[:10]

        return {
            "candidate_files": [
                {"path": f, "reason": "python_source", "score": 0.5}
                for f in candidate_files
            ],
            "tests": [f for f in files if "test" in f],
            "confidence": 0.6
        }


⸻

6. Symbol Locator Agent

coding/symbol_locator_agent.py

import re
from agents.micro_agents.base import BaseMicroAgent


class SymbolLocatorAgent(BaseMicroAgent):

    name = "symbol_locator"
    domain = "swe"

    async def run(self, task, context):

        issue = task.get("issue_text", "")

        symbols = re.findall(r"[A-Za-z_]{3,}", issue)

        return {
            "symbols": [{"name": s} for s in symbols[:5]]
        }


⸻

7. Dependency Tracer

coding/dependency_tracer_agent.py

from agents.micro_agents.base import BaseMicroAgent


class DependencyTracerAgent(BaseMicroAgent):

    name = "dependency_tracer"
    domain = "swe"

    async def run(self, task, context):

        symbols = context.get("symbols", [])

        return {
            "call_chain": [f"{s['name']} -> unknown_dependency" for s in symbols]
        }


⸻

8. Patch Planner

coding/patch_planner_agent.py

from agents.micro_agents.base import BaseMicroAgent


class PatchPlannerAgent(BaseMicroAgent):

    name = "patch_planner"
    domain = "swe"

    async def run(self, task, context):

        return {
            "plans": [
                {"id": "plan_a", "summary": "validate inputs before processing"},
                {"id": "plan_b", "summary": "add guard clause for null values"}
            ]
        }


⸻

9. Patch Writer

coding/patch_writer_agent.py

from agents.micro_agents.base import BaseMicroAgent


class PatchWriterAgent(BaseMicroAgent):

    name = "patch_writer"
    domain = "swe"

    async def run(self, task, context):

        plan = context.get("selected_plan", "plan_a")

        return {
            "plan_id": plan,
            "diff": "diff --git a/file.py b/file.py\n+ guard clause added",
            "changed_files": ["file.py"]
        }


⸻

10. Failure Analyzer

coding/failure_analyzer_agent.py

from agents.micro_agents.base import BaseMicroAgent


class FailureAnalyzerAgent(BaseMicroAgent):

    name = "failure_analyzer"
    domain = "swe"

    async def run(self, task, context):

        logs = context.get("test_logs", "")

        return {
            "failure_type": "logic_error",
            "reason": "function returned unexpected value"
        }


⸻

11. Patch Reviewer

coding/patch_reviewer_agent.py

from agents.micro_agents.base import BaseMicroAgent


class PatchReviewerAgent(BaseMicroAgent):

    name = "patch_reviewer"
    domain = "swe"

    async def run(self, task, context):

        diff = context.get("diff", "")

        return {
            "accept": True,
            "risks": [],
            "notes": "patch appears minimal"
        }


⸻

12. Runtime Registration Example

Add this during system startup.

from agents.micro_agents.registry import MicroAgentRegistry
from agents.micro_agents.coding.repo_mapper_agent import RepoMapperAgent
from agents.micro_agents.coding.symbol_locator_agent import SymbolLocatorAgent
from agents.micro_agents.coding.dependency_tracer_agent import DependencyTracerAgent
from agents.micro_agents.coding.patch_planner_agent import PatchPlannerAgent
from agents.micro_agents.coding.patch_writer_agent import PatchWriterAgent
from agents.micro_agents.coding.failure_analyzer_agent import FailureAnalyzerAgent
from agents.micro_agents.coding.patch_reviewer_agent import PatchReviewerAgent


registry = MicroAgentRegistry()

registry.register(RepoMapperAgent())
registry.register(SymbolLocatorAgent())
registry.register(DependencyTracerAgent())
registry.register(PatchPlannerAgent())
registry.register(PatchWriterAgent())
registry.register(FailureAnalyzerAgent())
registry.register(PatchReviewerAgent())


⸻

What This Gives You

You now have:
	•	a micro-agent framework
	•	a swarm controller
	•	structured schemas
	•	modular agent tasks

This converts a single reasoning chain into a collaborative reasoning system.

⸻

If you want, the next major upgrade would be building the consensus layer used in the strongest multi-agent systems, where multiple planners propose solutions and a judge model selects the best path before execution.

The consensus layer sits between planning and execution. Its purpose is to prevent the system from committing to the first solution proposed by one micro-agent. Instead, several planners propose candidate solutions and a separate evaluator ranks them.

This improves reliability because LLM outputs vary and individual planners often miss better options.

⸻

1. Where the Consensus Layer Lives

Add a directory inside the micro-agent orchestration layer.

agents/micro_agents/orchestration/
│
├── swarm_controller.py
├── consensus_engine.py
├── candidate_pool.py
└── solution_judge.py

This layer receives candidate plans from multiple agents, scores them, and returns the best option.

⸻

2. Conceptual Workflow

Instead of:

planner → execute patch

the flow becomes:

planner A
planner B
planner C
      ↓
candidate pool
      ↓
consensus engine
      ↓
solution judge
      ↓
best plan
      ↓
execution

Multiple planners explore different ideas. The judge chooses.

⸻

3. Candidate Pool

candidate_pool.py

Collects proposals from planners.

class CandidatePool:

    def __init__(self):
        self.candidates = []

    def add(self, candidate):
        self.candidates.append(candidate)

    def get_all(self):
        return self.candidates

    def clear(self):
        self.candidates = []

Candidate format:

{
  "plan_id": "plan_a",
  "summary": "...",
  "confidence": 0.73,
  "agent": "planner_agent_1"
}


⸻

4. Consensus Engine

consensus_engine.py

Aggregates proposals and filters weak candidates.

class ConsensusEngine:

    def __init__(self):
        pass

    def filter_candidates(self, candidates):

        scored = sorted(
            candidates,
            key=lambda c: c.get("confidence", 0),
            reverse=True
        )

        return scored[:3]

Only the strongest proposals continue to evaluation.

⸻

5. Solution Judge

solution_judge.py

Evaluates final candidates and selects the best one.

class SolutionJudge:

    def __init__(self):
        pass

    def select_best(self, candidates):

        best = None
        best_score = -1

        for c in candidates:
            score = c.get("confidence", 0)

            if score > best_score:
                best = c
                best_score = score

        return best

In a stronger system this step uses an LLM or policy model.

⸻

6. Example Multi-Planner Agents

You should run at least two planners.

patch_planner_agent_a
patch_planner_agent_b

Each produces different strategies.

Example output:

Planner A:

{
 "plan_id": "validate_empty_token",
 "summary": "add guard clause before decode",
 "confidence": 0.71
}

Planner B:

{
 "plan_id": "normalize_token_prefix",
 "summary": "strip bearer prefix before parsing",
 "confidence": 0.68
}

The consensus layer decides which plan is better.

⸻

7. Integrating Consensus Into the Swarm

Update the swarm controller.

Simplified flow:

plans = []

plans.append(await planner_a.run(task, context))
plans.append(await planner_b.run(task, context))

filtered = consensus_engine.filter_candidates(plans)

best_plan = solution_judge.select_best(filtered)

Execution continues with best_plan.

⸻

8. Improving the Judge

The judge can become more sophisticated.

Evaluation criteria:

factor	purpose
minimal patch size	reduce risk
dependency impact	avoid cascade failures
test coverage	maximize validation
historical success	use trajectory logs

Example scoring logic:

score =
0.4 * planner_confidence
+ 0.3 * predicted_test_success
+ 0.2 * patch_size_penalty
+ 0.1 * historical_success


⸻

9. Optional: Debate Mode

Some systems run planner debate before judgment.

planner A proposes solution
planner B critiques
planner A revises
judge decides

This improves patch quality but increases compute cost.

⸻

10. Logging the Consensus Process

Each decision should be logged.

Example record:

{
 "task_id": "...",
 "candidates": [
   {"agent": "planner_a", "score": 0.71},
   {"agent": "planner_b", "score": 0.68}
 ],
 "selected": "planner_a"
}

These logs feed the learning system.

⸻

11. Why Consensus Matters

Without consensus:
	•	the system executes the first idea
	•	errors propagate
	•	retry loops increase cost

With consensus:
	•	multiple hypotheses explored
	•	weaker ideas filtered early
	•	success rates increase

⸻

12. Resulting Agent Flow

Final reasoning pipeline:

task
 ↓
repo mapping agents
 ↓
dependency tracing
 ↓
multi-planner generation
 ↓
consensus engine
 ↓
solution judge
 ↓
patch generation
 ↓
test execution
 ↓
failure analysis


