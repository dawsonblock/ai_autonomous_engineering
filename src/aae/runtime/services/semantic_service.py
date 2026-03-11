from __future__ import annotations

from aae.code_analysis.call_signature_resolver import CallSignatureResolver
from aae.code_analysis.cfg_builder import CfgBuilder
from aae.code_analysis.type_inference import TypeInferenceEngine


class SemanticService:
    def __init__(
        self,
        cfg_builder: CfgBuilder | None = None,
        type_inference: TypeInferenceEngine | None = None,
        signature_resolver: CallSignatureResolver | None = None,
    ) -> None:
        self.cfg_builder = cfg_builder or CfgBuilder()
        self.type_inference = type_inference or TypeInferenceEngine()
        self.signature_resolver = signature_resolver or CallSignatureResolver()

    def build(self, repo_path: str, graph, graph_context: dict) -> dict:
        semantic_context = {}
        for entry in graph_context.get("symbol_context", []):
            for match in entry.get("matches", [])[:2]:
                summary = self.cfg_builder.build_for_symbol(
                    repo_path=repo_path,
                    file_path=match["path"],
                    symbol_id=match["id"],
                    qualname=match["qualname"],
                )
                inferred_types = self.type_inference.infer_for_function(
                    repo_path=repo_path,
                    file_path=match["path"],
                    function_name=match["name"],
                )
                resolved = self.signature_resolver.resolve(graph.snapshot, match["qualname"])
                semantic_context[match["name"]] = {
                    "cfg_nodes": summary.cfg_nodes,
                    "branch_points": summary.branch_points,
                    "inferred_types": inferred_types,
                    "signature": resolved.get("signature", ""),
                    "resolved_calls": resolved.get("resolved_calls", []),
                }
        return semantic_context
