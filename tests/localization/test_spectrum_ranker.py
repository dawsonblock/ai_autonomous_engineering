from aae.localization.models import CoverageRecord
from aae.localization.spectrum_ranker import SpectrumRanker


def test_spectrum_ranker():
    failed = [CoverageRecord(test_name="t1", file_path="main.py", line_hits=[1, 2, 3])]
    passed = [
        CoverageRecord(test_name="t2", file_path="main.py", line_hits=[1, 2]),
        CoverageRecord(test_name="t3", file_path="main.py", line_hits=[1]),
    ]
    ranker = SpectrumRanker()
    scores = ranker.score(failed, passed)

    assert round(scores[("main.py", 3)], 3) == 1.0
    assert round(scores[("main.py", 2)], 3) == 0.707
    assert round(scores[("main.py", 1)], 3) == 0.577
