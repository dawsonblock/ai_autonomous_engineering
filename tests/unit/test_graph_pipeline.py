from pathlib import Path
import shutil

from aae.graph.graph_query import GraphQueryEngine
from aae.graph.repo_graph_builder import RepoGraphBuilder
from aae.runtime.workspace import RepoMaterializer


FIXTURE_REPO = Path(__file__).resolve().parents[1] / "fixtures" / "sample_py_repo"


def test_repo_graph_builder_persists_sqlite_and_json(tmp_path: Path):
    builder = RepoGraphBuilder()
    build = builder.build(
        repo_path=str(FIXTURE_REPO),
        sqlite_path=str(tmp_path / "graph.sqlite3"),
        json_path=str(tmp_path / "graph.json"),
    )

    assert Path(build.sqlite_path).exists()
    assert Path(build.json_path).exists()
    assert build.stats["function_count"] >= 4

    graph = GraphQueryEngine.from_sqlite(build.sqlite_path)
    functions = graph.find_functions("authenticate").items
    assert any(item["path"] == "auth.py" for item in functions)

    paths = [" -> ".join(path) for path in graph.trace_call_chain("login").paths]
    assert any("api.login" in path and "auth.authenticate" in path for path in paths)

    tests = graph.tests_covering_function("authenticate").items
    assert any(item["path"] == "tests/test_auth.py" for item in tests)

    imports = graph.files_importing("auth").items
    assert any(item["path"] == "api.py" for item in imports)


def test_repo_materializer_creates_workflow_scoped_workspace(tmp_path: Path):
    materializer = RepoMaterializer(artifacts_dir=str(tmp_path))

    workspace = __import__("asyncio").run(
        materializer.materialize(workflow_id="wf_graph", repo_url=str(FIXTURE_REPO))
    )

    assert Path(workspace.repo_path).exists()
    assert workspace.repo_path.startswith(str((tmp_path / "workspaces" / "wf_graph").resolve()))


def test_repo_graph_builder_ignores_sandbox_artifacts(tmp_path: Path):
    repo_copy = tmp_path / "repo"
    shutil.copytree(FIXTURE_REPO, repo_copy)
    sandbox_file = repo_copy / ".sandbox_artifacts" / "local-sandbox-1" / "workspace" / "ghost.py"
    sandbox_file.parent.mkdir(parents=True, exist_ok=True)
    sandbox_file.write_text("def ghost():\n    return 'ignored'\n", encoding="utf-8")

    build = RepoGraphBuilder().build(
        repo_path=str(repo_copy),
        sqlite_path=str(tmp_path / "graph.sqlite3"),
        json_path=str(tmp_path / "graph.json"),
    )

    graph = GraphQueryEngine.from_sqlite(build.sqlite_path)
    assert not any(".sandbox_artifacts" in node.path for node in build.snapshot.nodes if node.path)
    assert not graph.find_functions("ghost").items
