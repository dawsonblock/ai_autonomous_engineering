import argparse

import pytest

from aae.runtime.system_launcher import validate_args


def parse_namespace(**overrides):
    defaults = {
        "workflow": "secure_build",
        "config": "configs/system_config.yaml",
        "query": "",
        "goal": "",
        "repo_url": "",
        "include_research": False,
        "include_post_audit": False,
    }
    defaults.update(overrides)
    return argparse.Namespace(**defaults)


def test_validate_args_requires_query_for_research_only():
    parser = argparse.ArgumentParser()
    args = parse_namespace(workflow="research_only")

    with pytest.raises(SystemExit):
        validate_args(parser, args)


def test_validate_args_requires_goal_and_repo_for_secure_build():
    parser = argparse.ArgumentParser()
    args = parse_namespace(workflow="secure_build")

    with pytest.raises(SystemExit):
        validate_args(parser, args)


def test_validate_args_accepts_valid_secure_build():
    parser = argparse.ArgumentParser()
    args = parse_namespace(
        workflow="secure_build",
        goal="Fix auth",
        repo_url="https://example.com/repo.git",
        include_research=True,
        query="Auth risks",
    )

    validate_args(parser, args)
