import { useState } from "react";
import { useMutation } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";

import { StatusBadge } from "@/components/status-badge";
import { api } from "@/lib/api";
import { validateLaunchDraft, type LaunchDraft } from "@/lib/validation";
import { useUiStore } from "@/state/ui-store";

const initialDraft: LaunchDraft = {
  workflow: "secure_build",
  query: "",
  goal: "",
  repo_url: "",
  include_research: false,
  include_post_audit: false,
};

export function LaunchWorkflowScreen() {
  const [draft, setDraft] = useState<LaunchDraft>(initialDraft);
  const [errors, setErrors] = useState<string[]>([]);
  const navigate = useNavigate();
  const rememberLaunch = useUiStore((state) => state.rememberLaunch);
  const recentLaunches = useUiStore((state) => state.recentLaunches);

  const mutation = useMutation({
    mutationFn: async () => api.launchWorkflow(draft),
    onSuccess: (detail) => {
      rememberLaunch(draft);
      navigate({ to: "/workflows/$workflowId", params: { workflowId: detail.summary.workflow_id } });
    },
    onError: (error) => {
      setErrors([error instanceof Error ? error.message : "Workflow launch failed."]);
    },
  });

  const submit = () => {
    const nextErrors = validateLaunchDraft(draft);
    setErrors(nextErrors);
    if (nextErrors.length > 0) {
      return;
    }
    mutation.mutate();
  };

  return (
    <div className="grid gap-4 xl:grid-cols-[1.15fr_0.85fr]">
      <section className="panel-muted space-y-5 p-5">
        <div className="flex items-start justify-between">
          <div>
            <h2 className="text-xl font-semibold">Launch Workflow</h2>
            <p className="mt-2 text-sm text-ink-600 dark:text-ink-200">
              Drive the embedded controller through the same workflow presets as the CLI.
            </p>
          </div>
          <StatusBadge value={draft.workflow} />
        </div>

        <div className="grid gap-4 md:grid-cols-2">
          <label className="space-y-2">
            <span className="text-sm font-medium">Workflow</span>
            <select
              className="w-full rounded-xl border border-ink-200 bg-white px-3 py-2 dark:border-white/10 dark:bg-white/5"
              value={draft.workflow}
              onChange={(event) => setDraft((current) => ({ ...current, workflow: event.target.value as LaunchDraft["workflow"] }))}
            >
              <option value="secure_build">secure_build</option>
              <option value="swe_only">swe_only</option>
              <option value="security_only">security_only</option>
              <option value="research_only">research_only</option>
            </select>
          </label>

          <label className="space-y-2">
            <span className="text-sm font-medium">Repository Path / URL</span>
            <input
              className="w-full rounded-xl border border-ink-200 bg-white px-3 py-2 dark:border-white/10 dark:bg-white/5"
              value={draft.repo_url}
              onChange={(event) => setDraft((current) => ({ ...current, repo_url: event.target.value }))}
              placeholder="/path/to/repo or git URL"
            />
          </label>
        </div>

        <label className="block space-y-2">
          <span className="text-sm font-medium">Goal</span>
          <textarea
            className="min-h-28 w-full rounded-xl border border-ink-200 bg-white px-3 py-2 dark:border-white/10 dark:bg-white/5"
            value={draft.goal}
            onChange={(event) => setDraft((current) => ({ ...current, goal: event.target.value }))}
            placeholder="Describe the repair goal or engineering task."
          />
        </label>

        <label className="block space-y-2">
          <span className="text-sm font-medium">Research Query</span>
          <input
            className="w-full rounded-xl border border-ink-200 bg-white px-3 py-2 dark:border-white/10 dark:bg-white/5"
            value={draft.query}
            onChange={(event) => setDraft((current) => ({ ...current, query: event.target.value }))}
            placeholder="Optional query for research_only or secure_build with research."
          />
        </label>

        <div className="flex flex-wrap gap-4 text-sm">
          <label className="inline-flex items-center gap-2">
            <input
              checked={draft.include_research}
              onChange={(event) => setDraft((current) => ({ ...current, include_research: event.target.checked }))}
              type="checkbox"
            />
            Include research
          </label>
          <label className="inline-flex items-center gap-2">
            <input
              checked={draft.include_post_audit}
              onChange={(event) => setDraft((current) => ({ ...current, include_post_audit: event.target.checked }))}
              type="checkbox"
            />
            Include post audit
          </label>
        </div>

        {errors.length > 0 ? (
          <div className="rounded-xl border border-signal-red/20 bg-signal-red/10 px-4 py-3 text-sm text-signal-red">
            {errors.map((error) => (
              <div key={error}>{error}</div>
            ))}
          </div>
        ) : null}

        <div className="flex items-center gap-3">
          <button
            type="button"
            onClick={submit}
            disabled={mutation.isPending}
            className="rounded-xl bg-ink-900 px-4 py-2.5 text-sm font-semibold text-white transition hover:bg-ink-700 disabled:opacity-60 dark:bg-white dark:text-ink-900"
          >
            {mutation.isPending ? "Launching…" : "Launch workflow"}
          </button>
          <button
            type="button"
            onClick={() => {
              setDraft(initialDraft);
              setErrors([]);
            }}
            className="rounded-xl border border-ink-200 px-4 py-2.5 text-sm font-medium dark:border-white/10"
          >
            Reset
          </button>
        </div>
      </section>

      <aside className="space-y-4">
        <section className="panel-muted p-5">
          <h3 className="text-lg font-semibold">Recent Inputs</h3>
          <div className="mt-4 space-y-3">
            {recentLaunches.length === 0 ? (
              <div className="text-sm text-ink-500 dark:text-ink-300">No recent launch presets saved yet.</div>
            ) : (
              recentLaunches.map((launch, index) => (
                <button
                  type="button"
                  key={`${launch.workflow}-${index}`}
                  onClick={() =>
                    setDraft({
                      workflow: String(launch.workflow) as LaunchDraft["workflow"],
                      goal: String(launch.goal ?? ""),
                      query: String(launch.query ?? ""),
                      repo_url: String(launch.repo_url ?? ""),
                      include_research: Boolean(launch.include_research),
                      include_post_audit: Boolean(launch.include_post_audit),
                    })
                  }
                  className="w-full rounded-xl border border-ink-200/80 px-4 py-3 text-left text-sm transition hover:border-signal-blue/40 dark:border-white/10"
                >
                  <div className="font-medium">{String(launch.workflow)}</div>
                  <div className="mt-1 truncate text-ink-500 dark:text-ink-300">{String(launch.goal ?? launch.query ?? "")}</div>
                </button>
              ))
            )}
          </div>
        </section>
      </aside>
    </div>
  );
}
