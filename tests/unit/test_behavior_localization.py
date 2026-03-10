import json
import sys
import threading
from pathlib import Path

import pytest

from aae.behavior_model.behavior_query_engine import BehaviorQueryEngine
from aae.behavior_model.state_graph_builder import StateGraphBuilder
from aae.behavior_model.state_transition_store import StateTransitionStore
from aae.behavior_model.trace_collector import TraceCollector
from aae.bug_localization.suspiciousness_ranker import SuspiciousnessRanker
from aae.contracts.behavior import TraceRecord
from aae.contracts.localization import FailureEvidence
from aae.graph.graph_query import GraphQueryEngine
from aae.graph.repo_graph_builder import RepoGraphBuilder


FIXTURE_REPO = Path(__file__).resolve().parents[1] / "fixtures" / "sample_py_repo"


def test_behavior_model_round_trip_and_queries(tmp_path: Path):
    build = RepoGraphBuilder().build(
        repo_path=str(FIXTURE_REPO),
        sqlite_path=str(tmp_path / "graph.sqlite3"),
        json_path=str(tmp_path / "graph.json"),
    )
    snapshot = StateGraphBuilder().build(str(FIXTURE_REPO), build.snapshot)
    store = StateTransitionStore(str(tmp_path / "behavior"))
    info = store.store_snapshot("wf_behavior", snapshot)
    store.append_traces(
        "wf_behavior",
        [
            TraceRecord(
                event_type="exception",
                function="authenticate",
                file_path=str(FIXTURE_REPO / "auth.py"),
                line=4,
                command_id="cmd-test",
                exception_type="ValueError",
            )
        ],
    )

    loaded_snapshot = store.load_snapshot("wf_behavior")
    assert loaded_snapshot is not None
    loaded_snapshot = loaded_snapshot.model_copy(update={"traces": store.load_traces("wf_behavior")})
    engine = BehaviorQueryEngine(loaded_snapshot)

    suspicious = engine.suspicious_files(["authenticate"])
    assert suspicious.items
    assert suspicious.items[0]["path"].endswith("auth.py")
    assert Path(info["snapshot_path"]).exists()


def test_trace_collector_records_call_return_and_exception(tmp_path: Path):
    module_path = tmp_path / "traced_module.py"
    module_path.write_text(
        "def sample(value):\n"
        "    if value < 0:\n"
        "        raise ValueError('bad input')\n"
        "    return value + 1\n",
        encoding="utf-8",
    )
    namespace = {}
    exec(compile(module_path.read_text(encoding="utf-8"), str(module_path), "exec"), namespace, namespace)

    trace_path = tmp_path / "trace.jsonl"
    collector = TraceCollector(str(trace_path), filter_root=str(tmp_path), command_id="cmd-trace", test_id="trace-test")
    collector.install()
    assert namespace["sample"](1) == 2
    with pytest.raises(ValueError):
        namespace["sample"](-1)
    sys.settrace(None)
    threading.settrace(None)
    collector.flush()

    records = [json.loads(line) for line in trace_path.read_text(encoding="utf-8").splitlines() if line.strip()]
    functions = {(record["event_type"], record["function"]) for record in records}
    assert ("call", "sample") in functions
    assert ("return", "sample") in functions
    assert ("exception", "sample") in functions


def test_suspiciousness_ranker_prefers_trace_and_failure_evidence(tmp_path: Path):
    build = RepoGraphBuilder().build(
        repo_path=str(FIXTURE_REPO),
        sqlite_path=str(tmp_path / "graph.sqlite3"),
        json_path=str(tmp_path / "graph.json"),
    )
    graph = GraphQueryEngine.from_sqlite(build.sqlite_path)
    behavior_snapshot = StateGraphBuilder().build(str(FIXTURE_REPO), build.snapshot).model_copy(
        update={
            "traces": [
                TraceRecord(
                    event_type="exception",
                    function="authenticate",
                    file_path=str(FIXTURE_REPO / "auth.py"),
                    line=4,
                    command_id="cmd-loc",
                    exception_type="ValueError",
                )
            ]
        }
    )
    behavior = BehaviorQueryEngine(behavior_snapshot)
    result = SuspiciousnessRanker().rank(
        graph=graph,
        behavior=behavior,
        candidate_symbols=["authenticate"],
        evidence=[
            FailureEvidence(
                source="stack_trace",
                file_path=str(FIXTURE_REPO / "auth.py"),
                symbol="authenticate",
                line=4,
                weight=0.9,
            )
        ],
    )

    assert result.suspicious_locations
    assert result.suspicious_locations[0].file_path.endswith("auth.py")
    assert "stack_trace" in result.suspicious_locations[0].evidence_sources
