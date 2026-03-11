export type LaunchDraft = {
  workflow: "research_only" | "security_only" | "swe_only" | "secure_build";
  query: string;
  goal: string;
  repo_url: string;
  include_research: boolean;
  include_post_audit: boolean;
};

export function validateLaunchDraft(draft: LaunchDraft): string[] {
  const errors: string[] = [];
  if (draft.workflow === "research_only" && !draft.query.trim()) {
    errors.push("Query is required for research workflows.");
  }
  if (draft.workflow === "security_only" && !draft.repo_url.trim()) {
    errors.push("Repository path or URL is required for security workflows.");
  }
  if ((draft.workflow === "swe_only" || draft.workflow === "secure_build") && !draft.goal.trim()) {
    errors.push("Goal is required for SWE workflows.");
  }
  if ((draft.workflow === "swe_only" || draft.workflow === "secure_build") && !draft.repo_url.trim()) {
    errors.push("Repository path or URL is required for SWE workflows.");
  }
  if (draft.workflow === "secure_build" && draft.include_research && !draft.query.trim()) {
    errors.push("Query is required when research is enabled for secure_build.");
  }
  return errors;
}
