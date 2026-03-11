import * as ScrollArea from "@radix-ui/react-scroll-area";
import * as Tabs from "@radix-ui/react-tabs";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useParams } from "@tanstack/react-router";
import React from "react";

import { JsonView } from "@/components/json-view";
import { StatusBadge } from "@/components/status-badge";
import { api } from "@/lib/api";
import { connectEvents } from "@/lib/sse";

export function WorkflowDetailScreen() {
  const { workflowId } = useParams({ strict: false });
  const queryClient = useQueryClient();
  const [liveEvents, setLiveEvents] = React.useState<Array<Record<string, unknown>>>([]);

  const detail = useQuery({
    queryKey: ["workflow", workflowId],
    queryFn: () => api.getWorkflow(workflowId!),
    enabled: Boolean(workflowId),
    refetchInterval: 5000,
  });

  React.useEffect(() => {
    if (!workflowId) return;
    return connectEvents(`/api/workflows/${workflowId}/events/stream`, (payload) => {
      setLiveEvents((current) => [payload, ...current].slice(0, 100));
      void queryClient.invalidateQueries({ queryKey: ["workflow", workflowId] });
      void queryClient.invalidateQueries({ queryKey: ["workflows"] });
    });
  }, [workflowId, queryClient]);

  const cancelMutation = useMutation({
    mutationFn: () => api.cancelWorkflow(workflowId!),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ["workflow", workflowId] });
      void queryClient.invalidateQueries({ queryKey: ["workflows"] });
    },
  });

  if (!workflowId) {
    return <div className="text-sm text-ink-500">Workflow id missing.</div>;
  }

  if (detail.isLoading) {
    return <div className="text-sm text-ink-500">Loading workflow detail…</div>;
  }

  if (detail.isError || !detail.data) {
    return <div className="text-sm text-signal-red">Unable to load workflow detail.</div>;
  }

  const summary = detail.data.summary;
  const memory = detail.data.memory_snapshot ?? {};
  const sandboxRuns = (memory.sandbox_runs as Array<Record<string, unknown>> | undefined) ?? [];
  const trustLevels = sandboxRuns
    .map((item) => String(item.trust_level ?? ""))
    .filter(Boolean)
    .slice(0, 3);

  return (
    <div className="space-y-5">
      <header className="flex flex-wrap items-start justify-between gap-4">
        <div>
          <div className="flex items-center gap-2">
            <StatusBadge value={summary.status} />
            {trustLevels.map((level) => (
              <StatusBadge key={level} value={level} />
            ))}
          </div>
          <h2 className="mt-3 text-2xl font-semibold">{summary.workflow_type}</h2>
          <div className="mt-2 font-mono text-sm text-ink-500 dark:text-ink-300">{summary.workflow_id}</div>
        </div>
        <button
          type="button"
          onClick={() => cancelMutation.mutate()}
          className="rounded-xl border border-signal-red/30 px-4 py-2.5 text-sm font-medium text-signal-red"
        >
          Cancel workflow
        </button>
      </header>

      <Tabs.Root defaultValue="timeline" className="space-y-4">
        <Tabs.List className="flex flex-wrap gap-2">
          {["timeline", "localization", "patch", "sandbox", "memory"].map((tab) => (
            <Tabs.Trigger
              key={tab}
              value={tab}
              className="rounded-xl border border-ink-200/80 px-3 py-2 text-sm font-medium data-[state=active]:bg-ink-900 data-[state=active]:text-white dark:border-white/10 dark:data-[state=active]:bg-white dark:data-[state=active]:text-ink-900"
            >
              {tab}
            </Tabs.Trigger>
          ))}
        </Tabs.List>

        <Tabs.Content value="timeline" className="panel-muted p-5">
          <ScrollArea.Root className="h-[420px] overflow-hidden rounded-xl">
            <ScrollArea.Viewport className="h-full w-full">
              <div className="space-y-3">
                {[...liveEvents, ...(detail.data.events ?? [])].slice(0, 120).map((event, index) => (
                  <div key={`${String(event.event_id ?? index)}-${index}`} className="rounded-xl border border-ink-200/70 px-4 py-3 dark:border-white/10">
                    <div className="flex items-center justify-between gap-4">
                      <div className="font-medium">{String(event.event_type ?? "event")}</div>
                      <div className="font-mono text-xs text-ink-500 dark:text-ink-300">{String(event.timestamp ?? "")}</div>
                    </div>
                    <div className="mt-2 text-sm text-ink-600 dark:text-ink-200">{String(event.task_id ?? "")}</div>
                  </div>
                ))}
              </div>
            </ScrollArea.Viewport>
          </ScrollArea.Root>
        </Tabs.Content>

        <Tabs.Content value="localization" className="panel-muted p-5">
          <JsonView value={memory.localization_ranked_context ?? memory.bug_localization ?? {}} />
        </Tabs.Content>

        <Tabs.Content value="patch" className="panel-muted p-5">
          <JsonView value={detail.data.planner.patch_provenance ?? detail.data.memory_snapshot.patch_provenance ?? []} />
        </Tabs.Content>

        <Tabs.Content value="sandbox" className="panel-muted space-y-4 p-5">
          {sandboxRuns.length === 0 ? (
            <div className="text-sm text-ink-500 dark:text-ink-300">No sandbox runs recorded yet.</div>
          ) : (
            sandboxRuns.map((run, index) => (
              <div key={`${run.command_id ?? index}`} className="rounded-xl border border-ink-200/70 p-4 dark:border-white/10">
                <div className="flex items-center gap-2">
                  <StatusBadge value={String(run.execution_mode ?? "local")} />
                  <StatusBadge value={String(run.trust_level ?? "degraded")} />
                </div>
                <div className="mt-3 text-sm text-ink-600 dark:text-ink-200">
                  {String(run.fallback_reason ?? run.patch_apply_status ?? "")}
                </div>
                <div className="mt-3 grid gap-3 md:grid-cols-2">
                  <div>
                    <div className="mb-2 text-xs font-semibold uppercase tracking-[0.16em] text-ink-500">stdout</div>
                    <pre className="code-block max-h-48">{String(run.stdout ?? "")}</pre>
                  </div>
                  <div>
                    <div className="mb-2 text-xs font-semibold uppercase tracking-[0.16em] text-ink-500">stderr</div>
                    <pre className="code-block max-h-48">{String(run.stderr ?? "")}</pre>
                  </div>
                </div>
              </div>
            ))
          )}
        </Tabs.Content>

        <Tabs.Content value="memory" className="panel-muted p-5">
          <JsonView value={memory} />
        </Tabs.Content>
      </Tabs.Root>
    </div>
  );
}
