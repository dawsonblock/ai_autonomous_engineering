from __future__ import annotations

import subprocess

from sec_af.diff_analysis import analyze_diff


def test_analyze_diff_collects_changed_and_blast_radius(monkeypatch) -> None:
    def _fake_run(command, **kwargs):
        _ = kwargs
        if command[:2] == ["git", "diff"]:
            return subprocess.CompletedProcess(command, 0, stdout="src/service/user.py\nREADME.md\n", stderr="")
        if command[:2] == ["git", "grep"]:
            return subprocess.CompletedProcess(
                command,
                0,
                stdout="HEAD:src/api/users.py\nHEAD:tests/test_users.py\n",
                stderr="",
            )
        raise AssertionError(f"unexpected command: {command}")

    monkeypatch.setattr("sec_af.diff_analysis.subprocess.run", _fake_run)

    analysis = analyze_diff("/tmp/repo", "base-sha", "head-sha")

    assert analysis.base_sha == "base-sha"
    assert analysis.head_sha == "head-sha"
    assert analysis.changed_files == ["src/service/user.py"]
    assert analysis.blast_radius_files == ["src/api/users.py"]
    assert analysis.all_relevant_files == ["src/api/users.py", "src/service/user.py"]
    assert analysis.file_count == 2


def test_analyze_diff_returns_empty_on_git_failure(monkeypatch) -> None:
    def _raise(*_args, **_kwargs):
        raise OSError("git not available")

    monkeypatch.setattr("sec_af.diff_analysis.subprocess.run", _raise)

    analysis = analyze_diff("/tmp/repo", "base-sha")

    assert analysis.base_sha == "base-sha"
    assert analysis.head_sha == "HEAD"
    assert analysis.changed_files == []
    assert analysis.blast_radius_files == []
    assert analysis.all_relevant_files == []
