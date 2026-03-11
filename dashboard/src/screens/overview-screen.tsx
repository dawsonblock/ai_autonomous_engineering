import { useQuery } from "@tanstack/react-query";
import { Link } from "@tanstack/react-router";

import { MetricCard } from "@/components/metric-card";
import { StatusBadge } from "@/components/status-badge";
import { api } from "@/lib/api";

export function OverviewScreen() {
  const workflows = useQuery({ queryKey: ["workflows"], queryFn: api.listWorkflows, refetchInterval: 4000 });
  const benchmarks = useQuery({ queryKey: ["benchmarks"], queryFn: api.benchmarkSummary, refetchInterval: 15000 });
  const diagnostics = useQuery({ queryKey: ["diagnostics"], queryFn: api.diagnostics, refetchInterval: 10000 });

  const latestMetrics = benchmarks.data?.latest?.metrics ?? {};

  return (
    <div className="space-y-6">
      <div className="grid gap-4 xl:grid-cols-4">
        <MetricCard label="Active Workflows" value={workflows.data?.filter((item) => item.status === "running" || item.status === "cancel_requested").length ?? 0} />
        <MetricCard label="Strict Fix Rate" value={latestMetrics.strict_fix_rate ?? 0} hint="Strict metric excludes degraded sandbox runs." />
        <MetricCard label="Raw Fix Rate" value={latestMetrics.raw_fix_rate ?? 0} hint="Raw metric shows all successes before trust gating." />
        <MetricCard label="Degraded Runs" value={latestMetrics.degraded_run_count ?? 0} hint="Local fallback runs never count as solved." />
      </div>

      <div className="grid gap-4 xl:grid-cols-[1.1fr_0.9fr]">
        <section className="panel-muted p-5">
          <div className="mb-4 flex items-center justify-between">
            <h2 className="text-lg font-semibold">Recent Workflows</h2>
            <Link to="/launch" className="text-sm font-medium text-signal-blue">
              Launch new
            </Link>
          </div>
          <div className="space-y-3">
            {(workflows.data ?? []).slice(0, 8).map((workflow) => (
              <Link
                key={workflow.workflow_id}
                to="/workflows/$workflowId"
                params={{ workflowId: workflow.workflow_id }}
                className="flex items-start justify-between rounded-xl border border-ink-200/70 px-4 py-3 transition hover:border-signal-blue/40 dark:border-white/10"
              >
                <div>
                  <div className="font-medium">{workflow.workflow_type}</div>
                  <div className="mt-1 font-mono text-xs text-ink-500 dark:text-ink-300">{workflow.workflow_id}</div>
                </div>
                <StatusBadge value={workflow.status} />
              </Link>
            ))}
          </div>
        </section>

        <section className="panel-muted p-5">
          <div className="mb-4 flex items-center justify-between">
            <h2 className="text-lg font-semibold">System Diagnostics</h2>
            <Link to="/settings" className="text-sm font-medium text-signal-blue">
              Runtime settings
            </Link>
          </div>
          <div className="space-y-3">
            {(diagnostics.data ?? []).map((diagnostic) => (
              <div key={diagnostic.name} className="rounded-xl border border-ink-200/70 px-4 py-3 dark:border-white/10">
                <div className="flex items-center justify-between">
                  <div className="font-medium">{diagnostic.name}</div>
                  <StatusBadge value={diagnostic.status} />
                </div>
                <div className="mt-2 text-sm text-ink-600 dark:text-ink-200">{diagnostic.summary}</div>
              </div>
            ))}
          </div>
        </section>
      </div>
    </div>
  );
}
