from __future__ import annotations

from aae.agents.micro_agents.base import BaseMicroAgent
from aae.contracts.micro_agents import PatchGenerationRequest, PatchPlan
from aae.patching.diff_constructor import DiffConstructor
from aae.patching.diff_optimizer import DiffOptimizer
from aae.patching.edit_locator import EditLocator
from aae.patching.patch_generator import HybridPatchGenerator
from aae.patching.patch_validator import PatchValidator


class PatchWriterAgent(BaseMicroAgent):
    name = "patch_writer"

    def __init__(
        self,
        edit_locator: EditLocator | None = None,
        patch_generator: HybridPatchGenerator | None = None,
        diff_constructor: DiffConstructor | None = None,
        diff_optimizer: DiffOptimizer | None = None,
        patch_validator: PatchValidator | None = None,
    ) -> None:
        self.edit_locator = edit_locator or EditLocator()
        self.patch_generator = patch_generator or HybridPatchGenerator()
        self.diff_constructor = diff_constructor or DiffConstructor()
        self.diff_optimizer = diff_optimizer or DiffOptimizer()
        self.patch_validator = patch_validator or PatchValidator()

    async def run(self, task, context):
        plan = PatchPlan.model_validate(context.get("selected_plan") or {})
        repo_path = context["repo_path"]
        semantic_context = context.get("semantic_context", {})
        graph_context = context.get("graph_context", {})
        graph_context = {
            **graph_context,
            "suspicious_locations": context.get("suspicious_locations", []),
        }
        target_spans = self.edit_locator.locate(repo_path, plan, semantic_context, graph_context)
        if not target_spans:
            return {
                "plan_id": plan.id,
                "diff": "",
                "changed_files": [],
                "confidence": float(plan.confidence),
                "target_spans": [],
                "syntax_valid": False,
                "constraint_results": [],
                "validation_errors": ["no target spans located"],
                "changed_symbols": [],
                "template_family": plan.template_family,
                "declared_intents": list(plan.declared_intents),
                "changed_line_count": 0,
                "provider_name": "none",
            }
        target_span = target_spans[0]
        repair_guidance = context.get("repair_guidance", {}) or plan.repair_guidance
        request = PatchGenerationRequest(
            file_path=target_span.file_path,
            symbol=target_span.symbol,
            strategy=plan.strategy,
            template_family=plan.template_family,
            expected_behavior=plan.summary,
            target_span=target_span,
            semantic_context=semantic_context.get(target_span.symbol, {}),
            constraints={"max_files": len(plan.target_files or [target_span.file_path])},
            suspicious_context={"suspicious_locations": context.get("suspicious_locations", [])[:4], "evidence": context.get("evidence", [])[:8]},
            repair_constraints=list(repair_guidance.get("constraints", [])),
            declared_allowed_intents=list(repair_guidance.get("declared_intents", plan.declared_intents)),
        )
        max_attempts = 3
        validation = None
        updated_text = ""
        diff = ""
        
        for attempt in range(max_attempts):
            original_text, llm_output = self.patch_generator.generate(repo_path, request)
            updated_text = self.diff_constructor.apply_llm_edits(target_span.file_path, original_text, llm_output)
            validation = self.patch_validator.validate(
                file_path=target_span.file_path,
                original_text=original_text,
                updated_text=updated_text,
                target_spans=target_spans,
                max_files=max(1, len(plan.target_files or [target_span.file_path])),
                max_changed_lines=20,
                declared_intents=list(repair_guidance.get("declared_intents", plan.declared_intents)),
            )
            
            # Check constraints
            failed_constraints = [res.details for res in validation.constraint_results if not res.passed]
            if validation.syntax_valid and not failed_constraints:
                diff = self.diff_constructor.build(target_span.file_path, original_text, updated_text)
                break
                
            # If failed, append feedback to prompt hint to try again
            feedback = "Your previous patch failed validation.\nErrors: %s\nConstraint failures: %s" % (
                ", ".join(validation.errors),
                ", ".join(failed_constraints)
            )
            # Update the prompt hint for the next iteration
            request.expected_behavior = "%s\n\n%s" % (plan.summary, feedback)

        # After max_attempts, validation contains the result of the last attempt
        if not diff and validation and validation.syntax_valid:
            diff = self.diff_constructor.build(target_span.file_path, original_text, updated_text)

        changed_line_count = self.diff_optimizer.changed_line_count(diff) if diff else 0
        return {
            "plan_id": plan.id,
            "diff": diff,
            "changed_files": [target_span.file_path] if diff else [],
            "confidence": float(plan.confidence),
            "target_spans": [span.model_dump(mode="json") for span in target_spans],
            "syntax_valid": validation.syntax_valid if validation else False,
            "constraint_results": [result.model_dump(mode="json") for result in validation.constraint_results] if validation else [],
            "validation_errors": list(validation.errors) if validation else ["No validation ran"],
            "changed_symbols": [target_span.symbol],
            "template_family": plan.template_family,
            "declared_intents": list(plan.declared_intents),
            "changed_line_count": changed_line_count,
            "provider_name": self.patch_generator.synthesizer.provider.__class__.__name__ if self.patch_generator.synthesizer.provider is not None else "deterministic_fallback",
            "repair_guidance": repair_guidance,
        }
