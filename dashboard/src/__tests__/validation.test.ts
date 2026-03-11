import { describe, expect, it } from "vitest";

import { validateLaunchDraft } from "@/lib/validation";

describe("validateLaunchDraft", () => {
  it("requires goal and repo path for secure_build", () => {
    const errors = validateLaunchDraft({
      workflow: "secure_build",
      goal: "",
      query: "",
      repo_url: "",
      include_research: false,
      include_post_audit: false,
    });

    expect(errors).toContain("Goal is required for SWE workflows.");
    expect(errors).toContain("Repository path or URL is required for SWE workflows.");
  });

  it("requires a query when research is enabled", () => {
    const errors = validateLaunchDraft({
      workflow: "secure_build",
      goal: "fix auth",
      query: "",
      repo_url: "/tmp/repo",
      include_research: true,
      include_post_audit: false,
    });

    expect(errors).toContain("Query is required when research is enabled for secure_build.");
  });
});
