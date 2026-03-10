from __future__ import annotations

import json
from pathlib import Path
from typing import Dict, Iterable, List

from aae.contracts.graph import CoverageAssociation, SymbolDefinition
from aae.graph.ast_parser import ParsedPythonFile


class CoverageMapper:
    def build(
        self,
        repo_path: str,
        parsed_files: Iterable[ParsedPythonFile],
        definitions: list[SymbolDefinition],
    ) -> list[CoverageAssociation]:
        by_name: Dict[str, list[SymbolDefinition]] = {}
        for definition in definitions:
            by_name.setdefault(definition.name, []).append(definition)
            by_name.setdefault(definition.qualname, []).append(definition)

        associations: List[CoverageAssociation] = []
        runtime_coverage = self._load_runtime_coverage(repo_path)
        for parsed in parsed_files:
            for test_node_id, target_names in parsed.test_targets.items():
                for target_name in target_names:
                    candidates = by_name.get(target_name, []) or by_name.get(target_name.split(".")[-1], [])
                    for candidate in candidates[:2]:
                        associations.append(
                            CoverageAssociation(
                                test_node_id=test_node_id,
                                target_symbol_id=candidate.symbol_id,
                                target_path=candidate.file_path,
                                source="static",
                                confidence=0.65,
                                metadata={"target_name": target_name},
                            )
                        )
        for item in runtime_coverage:
            associations.append(item)
        return associations

    def _load_runtime_coverage(self, repo_path: str) -> list[CoverageAssociation]:
        repo_root = Path(repo_path)
        for name in ["coverage_map.json", ".artifacts/coverage_map.json"]:
            path = repo_root / name
            if not path.exists():
                continue
            payload = json.loads(path.read_text(encoding="utf-8"))
            associations = []
            for row in payload.get("associations", []):
                associations.append(CoverageAssociation.model_validate({**row, "source": "runtime"}))
            return associations
        return []
