from aae.evaluation.experiment_evaluator import EvaluationMetric, ExperimentEvaluator


def test_evaluator_scores_passing_artifacts():
    evaluator = ExperimentEvaluator()
    result = evaluator.evaluate({
        "tests_passed": True,
        "lint_clean": True,
        "performance": True,
        "patch_minimal": True,
    })

    assert result.score == 1.0
    assert result.passed


def test_evaluator_scores_partial_artifacts():
    evaluator = ExperimentEvaluator()
    result = evaluator.evaluate({
        "tests_passed": True,
        "lint_clean": False,
        "performance": False,
        "patch_minimal": False,
    })

    assert 0 < result.score < 1.0


def test_evaluator_scores_zero_for_nothing():
    evaluator = ExperimentEvaluator()
    result = evaluator.evaluate({})

    assert result.score == 0.0
    assert not result.passed


def test_evaluator_custom_metrics():
    metrics = [
        EvaluationMetric("accuracy", weight=3.0),
        EvaluationMetric("speed", weight=1.0),
    ]
    evaluator = ExperimentEvaluator(metrics=metrics)
    result = evaluator.evaluate({"accuracy": True, "speed": False})

    assert result.score == 0.75  # 3.0 / 4.0
    assert result.passed


def test_evaluator_compare():
    evaluator = ExperimentEvaluator()
    result_a = evaluator.evaluate({"tests_passed": True, "lint_clean": True})
    result_b = evaluator.evaluate({"tests_passed": True, "lint_clean": False})

    assert evaluator.compare(result_a, result_b) == 1


def test_evaluator_pass_threshold():
    evaluator = ExperimentEvaluator(pass_threshold=0.8)
    result = evaluator.evaluate({
        "tests_passed": True,
        "lint_clean": True,
        "performance": False,
        "patch_minimal": False,
    })

    # 3.0/4.5 = 0.667 < 0.8
    assert not result.passed
