from aae.planner.long_horizon_planner import LongHorizonPlanner
from aae.planner.replanner import Replanner


def test_long_horizon_planner_fix_test():
    planner = LongHorizonPlanner()
    steps = planner.plan("fix failing test")

    assert len(steps) > 0
    actions = [step.action for step in steps]
    assert "analyze_code" in actions
    assert "run_tests" in actions


def test_long_horizon_planner_research():
    planner = LongHorizonPlanner()
    steps = planner.plan("research new approach")

    actions = [step.action for step in steps]
    assert "collect_sources" in actions
    assert "generate_hypothesis" in actions


def test_long_horizon_planner_feature():
    planner = LongHorizonPlanner()
    steps = planner.plan("implement new feature")

    actions = [step.action for step in steps]
    assert "write_code" in actions


def test_long_horizon_planner_as_graph():
    planner = LongHorizonPlanner()
    graph = planner.plan_as_graph("fix bug in tests")

    assert len(graph.nodes) > 0
    assert len(graph.edges) > 0
    ready = graph.get_ready()
    assert len(ready) > 0


def test_long_horizon_planner_dependencies():
    planner = LongHorizonPlanner()
    steps = planner.plan("fix failing test")

    step_map = {step.step_id: step for step in steps}
    assert step_map["locate"].depends_on == ["analyze"]
    assert step_map["design"].depends_on == ["locate"]


def test_replanner_success():
    replanner = Replanner()
    decision = replanner.revise({"success": True})

    assert decision.action == "continue"


def test_replanner_test_failure():
    replanner = Replanner()
    decision = replanner.revise({"success": False, "step": "run_tests", "error": "test failed"})

    assert decision.action == "retry_with_fix"
    assert len(decision.alternative_steps) > 0


def test_replanner_patch_failure():
    replanner = Replanner()
    decision = replanner.revise({"success": False, "step": "apply_patch", "error": "conflict"})

    assert decision.action == "generate_alternative"


def test_replanner_timeout():
    replanner = Replanner()
    decision = replanner.revise({"success": False, "step": "analyze", "error": "timeout occurred"})

    assert decision.action == "retry"


def test_replanner_should_retry():
    replanner = Replanner()
    assert replanner.should_retry({"success": False}, attempt=1, max_attempts=3)
    assert not replanner.should_retry({"success": False}, attempt=3, max_attempts=3)
    assert not replanner.should_retry({"success": True}, attempt=1, max_attempts=3)
