from __future__ import annotations


class DiffOptimizer:
    def changed_line_count(self, diff_text: str) -> int:
        count = 0
        for line in diff_text.splitlines():
            if line.startswith(("+++", "---", "@@")):
                continue
            if line.startswith("+") or line.startswith("-"):
                count += 1
        return count
