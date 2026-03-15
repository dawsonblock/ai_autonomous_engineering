from aae.core.task_graph import ActionGraph, ActionState


def test_action_graph_initial_state():
    graph = ActionGraph()
    graph.add_task("analyze")
    graph.add_task("patch")
    graph.add_edge("analyze", "patch")

    assert graph.get_state("analyze") == ActionState.READY
    assert graph.get_state("patch") == ActionState.PENDING


def test_action_graph_marks_dependents_ready():
    graph = ActionGraph()
    graph.add_task("a")
    graph.add_task("b")
    graph.add_task("c")
    graph.add_edge("a", "b")
    graph.add_edge("b", "c")

    ready = graph.get_ready()
    assert ready == ["a"]

    newly_ready = graph.mark_done("a")
    assert "b" in newly_ready
    assert graph.get_state("b") == ActionState.READY

    graph.mark_done("b")
    assert graph.get_state("c") == ActionState.READY


def test_action_graph_topological_order():
    graph = ActionGraph()
    graph.add_task("analyze")
    graph.add_task("patch")
    graph.add_task("test")
    graph.add_edge("analyze", "patch")
    graph.add_edge("patch", "test")

    order = graph.topological_order()
    assert order.index("analyze") < order.index("patch")
    assert order.index("patch") < order.index("test")


def test_action_graph_parallel_dependencies():
    graph = ActionGraph()
    graph.add_task("analyze")
    graph.add_task("testA")
    graph.add_task("testB")
    graph.add_task("merge")
    graph.add_edge("analyze", "testA")
    graph.add_edge("analyze", "testB")
    graph.add_edge("testA", "merge")
    graph.add_edge("testB", "merge")

    graph.mark_done("analyze")
    ready = graph.get_ready()
    assert set(ready) == {"testA", "testB"}

    graph.mark_done("testA")
    assert graph.get_state("merge") == ActionState.PENDING

    graph.mark_done("testB")
    assert graph.get_state("merge") == ActionState.READY


def test_action_graph_all_done():
    graph = ActionGraph()
    graph.add_task("a")
    graph.add_task("b")
    graph.add_edge("a", "b")

    assert not graph.all_done()

    graph.mark_done("a")
    assert not graph.all_done()

    graph.mark_done("b")
    assert graph.all_done()


def test_action_graph_mark_failed():
    graph = ActionGraph()
    graph.add_task("a")
    graph.mark_failed("a")
    assert graph.get_state("a") == ActionState.FAILED
