from pathlib import Path

from aae.memory.repo_model import DependencyGraph, FileIndex, FileEntry, RepoModel, SymbolGraph, SymbolInfo


def test_file_index_scans_directory(tmp_path: Path):
    (tmp_path / "app.py").write_text("print('hello')")
    (tmp_path / "utils.py").write_text("def helper(): pass")
    (tmp_path / "README.md").write_text("# Project")

    index = FileIndex()
    index.scan(str(tmp_path))

    assert index.count == 3
    entry = index.get("app.py")
    assert entry is not None
    assert entry.language == "python"


def test_file_index_ignores_hidden_dirs(tmp_path: Path):
    (tmp_path / ".git").mkdir()
    (tmp_path / ".git" / "config").write_text("git config")
    (tmp_path / "app.py").write_text("code")

    index = FileIndex()
    index.scan(str(tmp_path))

    assert index.count == 1


def test_file_index_by_language(tmp_path: Path):
    (tmp_path / "a.py").write_text("pass")
    (tmp_path / "b.py").write_text("pass")
    (tmp_path / "c.js").write_text("//")

    index = FileIndex()
    index.scan(str(tmp_path))

    python_files = index.by_language("python")
    assert len(python_files) == 2


def test_symbol_graph_tracks_calls():
    graph = SymbolGraph()
    graph.add_symbol(SymbolInfo(name="foo", kind="function", file_path="a.py"))
    graph.add_symbol(SymbolInfo(name="bar", kind="function", file_path="a.py"))
    graph.add_call("foo", "bar")

    assert "foo" in graph.callers_of("bar")
    assert "bar" in graph.callees_of("foo")


def test_symbol_graph_impacted_by():
    graph = SymbolGraph()
    graph.add_symbol(SymbolInfo(name="base", kind="function", file_path="a.py"))
    graph.add_symbol(SymbolInfo(name="middle", kind="function", file_path="b.py"))
    graph.add_symbol(SymbolInfo(name="top", kind="function", file_path="c.py"))
    graph.add_call("middle", "base")
    graph.add_call("top", "middle")

    impacted = graph.impacted_by("base")
    assert "middle" in impacted
    assert "top" in impacted


def test_dependency_graph_tracks_imports():
    graph = DependencyGraph()
    graph.add_import("app.py", "utils")
    graph.add_import("app.py", "models")
    graph.add_import("tests/test_app.py", "app")

    assert "utils" in graph.dependencies_of("app.py")
    assert "app.py" in graph.dependents_of("utils")


def test_dependency_graph_transitive_dependents():
    graph = DependencyGraph()
    graph.add_import("b.py", "a")
    graph.add_import("c.py", "b.py")

    dependents = graph.transitive_dependents("a")
    assert "b.py" in dependents
    assert "c.py" in dependents


def test_repo_model_update_from_repo(tmp_path: Path):
    (tmp_path / "main.py").write_text("import utils\n")
    (tmp_path / "utils.py").write_text("def helper(): pass\n")

    model = RepoModel()
    stats = model.update_from_repo(str(tmp_path))

    assert stats["file_count"] == 2


def test_repo_model_impacted_tests():
    model = RepoModel()
    model.files.files["src/utils.py"] = FileEntry(path="src/utils.py", language="python", size_bytes=100)
    model.files.files["tests/test_utils.py"] = FileEntry(path="tests/test_utils.py", language="python", size_bytes=100)
    model.dependencies.add_import("tests/test_utils.py", "src/utils.py")

    tests = model.impacted_tests("src/utils.py")
    assert "tests/test_utils.py" in tests
