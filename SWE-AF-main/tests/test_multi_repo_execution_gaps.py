"""Tests for multi-repo execution gap fixes.

Validates that workspace_manifest and target_repo flow through the inner
execution loops (coder, QA, reviewer, synthesizer, integration tester) and
that worktree setup routes correctly per-repo.
"""

from __future__ import annotations

import pytest

from swe_af.execution.schemas import WorkspaceManifest, WorkspaceRepo

# ---------------------------------------------------------------------------
# Fixtures
# ---------------------------------------------------------------------------

SAMPLE_REPOS = [
    WorkspaceRepo(
        repo_name="backend",
        repo_url="https://github.com/org/backend",
        role="primary",
        absolute_path="/workspace/backend",
        branch="main",
    ),
    WorkspaceRepo(
        repo_name="frontend",
        repo_url="https://github.com/org/frontend",
        role="dependency",
        absolute_path="/workspace/frontend",
        branch="main",
    ),
]

SAMPLE_MANIFEST = WorkspaceManifest(
    workspace_root="/workspace",
    repos=SAMPLE_REPOS,
    primary_repo_name="backend",
)


# ---------------------------------------------------------------------------
# Step 1: Prompt functions accept workspace params
# ---------------------------------------------------------------------------


class TestPromptWorkspaceParams:
    """Verify all prompt functions accept workspace_manifest and produce correct output."""

    def test_qa_prompt_none_manifest(self):
        from swe_af.prompts.qa import qa_task_prompt

        result = qa_task_prompt(
            worktree_path="/repo",
            coder_result={"summary": "done", "files_changed": []},
            issue={"name": "test-issue", "title": "Test"},
        )
        assert "Workspace Repositories" not in result

    def test_qa_prompt_with_manifest(self):
        from swe_af.prompts.qa import qa_task_prompt

        result = qa_task_prompt(
            worktree_path="/repo",
            coder_result={"summary": "done", "files_changed": []},
            issue={"name": "test-issue", "title": "Test"},
            workspace_manifest=SAMPLE_MANIFEST,
            target_repo="backend",
        )
        assert "Workspace Repositories" in result
        assert "backend" in result
        assert "frontend" in result
        assert "Target Repository: `backend`" in result

    def test_code_reviewer_prompt_none_manifest(self):
        from swe_af.prompts.code_reviewer import code_reviewer_task_prompt

        result = code_reviewer_task_prompt(
            worktree_path="/repo",
            coder_result={"summary": "done", "files_changed": []},
            issue={"name": "test-issue", "title": "Test"},
        )
        assert "Workspace Repositories" not in result

    def test_code_reviewer_prompt_with_manifest(self):
        from swe_af.prompts.code_reviewer import code_reviewer_task_prompt

        result = code_reviewer_task_prompt(
            worktree_path="/repo",
            coder_result={"summary": "done", "files_changed": []},
            issue={"name": "test-issue", "title": "Test"},
            workspace_manifest=SAMPLE_MANIFEST,
            target_repo="frontend",
        )
        assert "Workspace Repositories" in result
        assert "Target Repository: `frontend`" in result

    def test_qa_synthesizer_prompt_none_manifest(self):
        from swe_af.prompts.qa_synthesizer import qa_synthesizer_task_prompt

        result = qa_synthesizer_task_prompt(
            qa_result={"passed": True, "summary": "ok"},
            review_result={"approved": True, "summary": "ok"},
            iteration_history=[],
        )
        assert "Workspace Repositories" not in result

    def test_qa_synthesizer_prompt_with_manifest(self):
        from swe_af.prompts.qa_synthesizer import qa_synthesizer_task_prompt

        result = qa_synthesizer_task_prompt(
            qa_result={"passed": True, "summary": "ok"},
            review_result={"approved": True, "summary": "ok"},
            iteration_history=[],
            workspace_manifest=SAMPLE_MANIFEST,
        )
        assert "Workspace Repositories" in result

    def test_integration_tester_prompt_none_manifest(self):
        from swe_af.prompts.integration_tester import integration_tester_task_prompt

        result = integration_tester_task_prompt(
            repo_path="/repo",
            integration_branch="integration/build-1",
            merged_branches=[],
            prd_summary="Build stuff",
            architecture_summary="Simple arch",
            conflict_resolutions=[],
        )
        assert "Workspace Repositories" not in result

    def test_integration_tester_prompt_with_manifest(self):
        from swe_af.prompts.integration_tester import integration_tester_task_prompt

        result = integration_tester_task_prompt(
            repo_path="/repo",
            integration_branch="integration/build-1",
            merged_branches=[],
            prd_summary="Build stuff",
            architecture_summary="Simple arch",
            conflict_resolutions=[],
            workspace_manifest=SAMPLE_MANIFEST,
        )
        assert "Workspace Repositories" in result
        assert "backend" in result
        assert "frontend" in result


# ---------------------------------------------------------------------------
# Step 2: Execution agent reasoners accept workspace params
# ---------------------------------------------------------------------------


class TestReasonerSignatures:
    """Verify execution agent reasoner functions accept workspace params."""

    def test_maybe_workspace_manifest_none(self):
        from swe_af.reasoners.execution_agents import _maybe_workspace_manifest

        assert _maybe_workspace_manifest(None) is None

    def test_maybe_workspace_manifest_dict(self):
        from swe_af.reasoners.execution_agents import _maybe_workspace_manifest

        result = _maybe_workspace_manifest(SAMPLE_MANIFEST.model_dump())
        assert isinstance(result, WorkspaceManifest)
        assert result.primary_repo_name == "backend"
        assert len(result.repos) == 2

    def test_run_coder_accepts_workspace_params(self):
        """run_coder signature accepts workspace_manifest and target_repo."""
        import inspect
        from swe_af.reasoners.execution_agents import run_coder

        sig = inspect.signature(run_coder)
        assert "workspace_manifest" in sig.parameters
        assert "target_repo" in sig.parameters
        assert sig.parameters["workspace_manifest"].default is None
        assert sig.parameters["target_repo"].default == ""

    def test_run_qa_accepts_workspace_params(self):
        import inspect
        from swe_af.reasoners.execution_agents import run_qa

        sig = inspect.signature(run_qa)
        assert "workspace_manifest" in sig.parameters
        assert "target_repo" in sig.parameters

    def test_run_code_reviewer_accepts_workspace_params(self):
        import inspect
        from swe_af.reasoners.execution_agents import run_code_reviewer

        sig = inspect.signature(run_code_reviewer)
        assert "workspace_manifest" in sig.parameters
        assert "target_repo" in sig.parameters

    def test_run_qa_synthesizer_accepts_workspace_manifest(self):
        import inspect
        from swe_af.reasoners.execution_agents import run_qa_synthesizer

        sig = inspect.signature(run_qa_synthesizer)
        assert "workspace_manifest" in sig.parameters

    def test_run_integration_tester_accepts_workspace_manifest(self):
        import inspect
        from swe_af.reasoners.execution_agents import run_integration_tester

        sig = inspect.signature(run_integration_tester)
        assert "workspace_manifest" in sig.parameters


# ---------------------------------------------------------------------------
# Step 3: Coding loop passes workspace context
# ---------------------------------------------------------------------------


class TestCodingLoopWiring:
    """Verify coding loop extracts and passes workspace context to agents."""

    def test_run_default_path_accepts_workspace_params(self):
        import inspect
        from swe_af.execution.coding_loop import _run_default_path

        sig = inspect.signature(_run_default_path)
        assert "workspace_manifest" in sig.parameters
        assert "target_repo" in sig.parameters

    def test_run_flagged_path_accepts_workspace_params(self):
        import inspect
        from swe_af.execution.coding_loop import _run_flagged_path

        sig = inspect.signature(_run_flagged_path)
        assert "workspace_manifest" in sig.parameters
        assert "target_repo" in sig.parameters


# ---------------------------------------------------------------------------
# Step 4: Worktree setup enrichment helper
# ---------------------------------------------------------------------------


class TestWorktreeEnrichment:
    """Verify the _enrich_issues_from_setup helper works correctly."""

    def test_enrich_issues_maps_worktrees(self):
        from swe_af.execution.dag_executor import _enrich_issues_from_setup

        issues = [
            {"name": "add-auth", "title": "Add auth"},
            {"name": "fix-ui", "title": "Fix UI"},
        ]
        setup = {
            "success": True,
            "workspaces": [
                {"issue_name": "add-auth", "worktree_path": "/repo/.worktrees/add-auth", "branch_name": "issue/01-add-auth"},
                {"issue_name": "fix-ui", "worktree_path": "/repo/.worktrees/fix-ui", "branch_name": "issue/02-fix-ui"},
            ],
        }

        result = _enrich_issues_from_setup(issues, setup, "integration/build-1")
        assert len(result) == 2
        assert result[0]["worktree_path"] == "/repo/.worktrees/add-auth"
        assert result[0]["branch_name"] == "issue/01-add-auth"
        assert result[0]["integration_branch"] == "integration/build-1"
        assert result[1]["worktree_path"] == "/repo/.worktrees/fix-ui"

    def test_enrich_issues_handles_missing(self):
        from swe_af.execution.dag_executor import _enrich_issues_from_setup

        issues = [{"name": "unknown-issue", "title": "Unknown"}]
        setup = {"success": True, "workspaces": []}

        result = _enrich_issues_from_setup(issues, setup, "integration/build-1")
        assert len(result) == 1
        assert "worktree_path" not in result[0]

    def test_enrich_issues_handles_sequence_prefix(self):
        from swe_af.execution.dag_executor import _enrich_issues_from_setup

        issues = [{"name": "add-auth", "title": "Add auth"}]
        setup = {
            "success": True,
            "workspaces": [
                {"issue_name": "01-add-auth", "worktree_path": "/repo/.worktrees/01-add-auth", "branch_name": "issue/01-add-auth"},
            ],
        }

        result = _enrich_issues_from_setup(issues, setup, "integration/build-1")
        assert result[0]["worktree_path"] == "/repo/.worktrees/01-add-auth"


# ---------------------------------------------------------------------------
# Step 5: Integration test routing
# ---------------------------------------------------------------------------


class TestIntegrationTestRouting:
    """Verify integration test passes workspace_manifest to the agent."""

    def test_run_integration_tests_signature_unchanged(self):
        """_run_integration_tests still works with single-repo (no extra params needed)."""
        import inspect
        from swe_af.execution.dag_executor import _run_integration_tests

        sig = inspect.signature(_run_integration_tests)
        # Same params as before — workspace_manifest comes from dag_state
        assert "dag_state" in sig.parameters
        assert "merge_result" in sig.parameters


# ---------------------------------------------------------------------------
# Backward compatibility
# ---------------------------------------------------------------------------


class TestBackwardCompatibility:
    """Verify single-repo callers are unaffected (all new params have defaults)."""

    def test_qa_prompt_backward_compat(self):
        from swe_af.prompts.qa import qa_task_prompt

        # Call with original signature (no new params)
        result = qa_task_prompt(
            worktree_path="/repo",
            coder_result={"summary": "done"},
            issue={"name": "x", "title": "X"},
            iteration_id="abc",
            project_context={"prd_path": "/prd"},
        )
        assert "Issue Under Test" in result
        assert "Workspace" not in result

    def test_code_reviewer_prompt_backward_compat(self):
        from swe_af.prompts.code_reviewer import code_reviewer_task_prompt

        result = code_reviewer_task_prompt(
            worktree_path="/repo",
            coder_result={"summary": "done"},
            issue={"name": "x", "title": "X"},
        )
        assert "Issue Under Review" in result
        assert "Workspace" not in result

    def test_qa_synthesizer_prompt_backward_compat(self):
        from swe_af.prompts.qa_synthesizer import qa_synthesizer_task_prompt

        result = qa_synthesizer_task_prompt(
            qa_result={"passed": True, "summary": "ok"},
            review_result={"approved": True, "summary": "ok"},
            iteration_history=[],
        )
        assert "Issue Being Evaluated" not in result or "QA Results" in result
        assert "Workspace" not in result

    def test_integration_tester_prompt_backward_compat(self):
        from swe_af.prompts.integration_tester import integration_tester_task_prompt

        result = integration_tester_task_prompt(
            repo_path="/repo",
            integration_branch="main",
            merged_branches=[],
            prd_summary="prd",
            architecture_summary="arch",
            conflict_resolutions=[],
        )
        assert "Integration Testing Task" in result
        assert "Workspace" not in result
