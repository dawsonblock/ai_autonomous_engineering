from __future__ import annotations

from aae.contracts.planner import BranchComparisonResult


class ResultComparator:
    def compare(self, results: list[dict]) -> BranchComparisonResult:
        rankings = []
        for result in results:
            execution = result.get("execution", {})
            patch = result.get("patch_candidate", {})
            tests_passed = int(execution.get("tests_passed", execution.get("tests_passed", 0)) or 0)
            tests_failed = int(execution.get("tests_failed", execution.get("regression_count", 0)) or 0)
            patch_size = int(patch.get("changed_line_count", 0) or 0)
            risk_score = float(patch.get("simulation", {}).get("risk_score", 0.0))
            score = (tests_passed * 1.0) - (tests_failed * 1.2) - (patch_size * 0.03) - (risk_score * 0.5)
            rankings.append(
                {
                    "branch_id": result.get("branch_id", ""),
                    "score": round(score, 3),
                    "tests_passed": tests_passed,
                    "tests_failed": tests_failed,
                    "patch_size": patch_size,
                    "risk_score": risk_score,
                }
            )
        rankings.sort(key=lambda item: item["score"], reverse=True)
        return BranchComparisonResult(
            selected_branch_id=rankings[0]["branch_id"] if rankings else "",
            rankings=rankings,
            summary={"result_count": len(rankings)},
        )
