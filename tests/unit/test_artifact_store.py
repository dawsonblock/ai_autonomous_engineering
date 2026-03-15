from pathlib import Path

from aae.memory.artifacts.artifact_store import Artifact, ArtifactStore


def test_artifact_store_records_and_retrieves(tmp_path: Path):
    store = ArtifactStore(base_dir=str(tmp_path / "artifacts"))
    artifact = Artifact(
        artifact_type="patch",
        data={"file": "utils.py", "change": "fix recursion bug"},
        task_id="task_42",
    )
    aid = store.record(artifact)

    retrieved = store.get(aid)
    assert retrieved is not None
    assert retrieved.artifact_type == "patch"
    assert retrieved.task_id == "task_42"


def test_artifact_store_create_shortcut(tmp_path: Path):
    store = ArtifactStore(base_dir=str(tmp_path / "artifacts"))
    artifact = store.create(
        artifact_type="report",
        data={"content": "analysis results"},
        task_id="task_1",
    )

    assert store.count == 1
    assert artifact.artifact_type == "report"


def test_artifact_store_by_task(tmp_path: Path):
    store = ArtifactStore(base_dir=str(tmp_path / "artifacts"))
    store.create(artifact_type="patch", data={"diff": "..."}, task_id="t1")
    store.create(artifact_type="report", data={"text": "..."}, task_id="t1")
    store.create(artifact_type="patch", data={"diff": "..."}, task_id="t2")

    t1_artifacts = store.by_task("t1")
    assert len(t1_artifacts) == 2


def test_artifact_store_by_type(tmp_path: Path):
    store = ArtifactStore(base_dir=str(tmp_path / "artifacts"))
    store.create(artifact_type="patch", data={}, task_id="t1")
    store.create(artifact_type="report", data={}, task_id="t1")
    store.create(artifact_type="patch", data={}, task_id="t2")

    patches = store.by_type("patch")
    assert len(patches) == 2


def test_artifact_store_persists_to_disk(tmp_path: Path):
    store = ArtifactStore(base_dir=str(tmp_path / "artifacts"))
    artifact = store.create(artifact_type="benchmark", data={"score": 0.95})

    stored_path = tmp_path / "artifacts" / ("%s.json" % artifact.artifact_id)
    assert stored_path.exists()


def test_artifact_linked_to_task_and_evaluation(tmp_path: Path):
    store = ArtifactStore(base_dir=str(tmp_path / "artifacts"))
    patch = store.create(
        artifact_type="patch",
        data={"file": "utils.py"},
        task_id="task_42",
        metadata={"evaluation_score": 0.82},
    )

    assert patch.metadata["evaluation_score"] == 0.82
    assert patch.task_id == "task_42"
