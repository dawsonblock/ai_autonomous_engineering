from aae.contracts.tasks import TaskSpec, TaskState
from aae.contracts.workflow import WorkflowSpec
from aae.controller.task_graph import TaskGraph


def test_task_graph_unlocks_ready_tasks_after_success():
    workflow = WorkflowSpec(
        workflow_id="wf",
        workflow_type="test",
        tasks=[
            TaskSpec(task_id="a", task_type="research", agent_name="deep_research"),
            TaskSpec(
                task_id="b",
                task_type="security_audit",
                agent_name="sec_af",
                depends_on=["a"],
            ),
        ],
    )

    graph = TaskGraph(workflow)

    assert graph.get_state("a") == TaskState.READY
    assert graph.get_state("b") == TaskState.PENDING

    updates = graph.mark_succeeded("a")

    assert updates["ready"] == ["b"]
    assert graph.get_state("b") == TaskState.READY


def test_task_graph_blocks_hard_dependents_after_failure():
    workflow = WorkflowSpec(
        workflow_id="wf",
        workflow_type="test",
        tasks=[
            TaskSpec(task_id="security_baseline", task_type="security_audit", agent_name="sec_af"),
            TaskSpec(
                task_id="swe_build",
                task_type="swe_build",
                agent_name="swe_af",
                depends_on=["security_baseline"],
            ),
            TaskSpec(
                task_id="security_post",
                task_type="security_audit",
                agent_name="sec_af",
                depends_on=["swe_build"],
            ),
        ],
    )

    graph = TaskGraph(workflow)
    updates = graph.mark_failed("security_baseline", reason="audit failed")

    assert "swe_build" in updates["blocked"]
    assert "security_post" in updates["blocked"]
    assert graph.get_state("swe_build") == TaskState.BLOCKED
    assert graph.get_state("security_post") == TaskState.BLOCKED


def test_task_graph_allows_soft_dependency_failure_once_terminal():
    workflow = WorkflowSpec(
        workflow_id="wf",
        workflow_type="test",
        tasks=[
            TaskSpec(task_id="research", task_type="research", agent_name="deep_research"),
            TaskSpec(task_id="security_baseline", task_type="security_audit", agent_name="sec_af"),
            TaskSpec(
                task_id="swe_build",
                task_type="swe_build",
                agent_name="swe_af",
                depends_on=["security_baseline", "research"],
                soft_dependencies=["research"],
            ),
        ],
    )

    graph = TaskGraph(workflow)
    graph.mark_succeeded("security_baseline")
    updates = graph.mark_failed("research", reason="research failed")

    assert updates["ready"] == ["swe_build"]
    assert graph.get_state("swe_build") == TaskState.READY
