import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { MetricCard } from "@/components/metric-card";
import { StatusBadge } from "@/components/status-badge";
import { api } from "@/lib/api";

export function BenchmarksScreen() {
  const queryClient = useQueryClient();
  const benchmarks = useQuery({ queryKey: ["benchmarks"], queryFn: api.benchmarkSummary, refetchInterval: 10000 });
  const runMutation = useMutation({
    mutationFn: api.runBenchmarks,
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ["benchmarks"] });
    },
  });

  const metrics = benchmarks.data?.latest?.metrics ?? {};

  return (
    <div className="space-y-6">
      <div className="flex items-start justify-between gap-4">
        <div>
          <h2 className="text-2xl font-semibold">Benchmarks</h2>
          <p className="mt-2 text-sm text-ink-600 dark:text-ink-200">
            Strict metrics exclude degraded local fallback runs. Raw metrics stay visible for debugging.
          </p>
        </div>
        <button
          type="button"
          onClick={() => runMutation.mutate()}
          disabled={runMutation.isPending}
          className="rounded-xl bg-ink-900 px-4 py-2.5 text-sm font-semibold text-white dark:bg-white dark:text-ink-900"
        >
          {runMutation.isPending ? "Running…" : "Run corpus"}
        </button>
      </div>

      <div className="grid gap-4 xl:grid-cols-5">
        <MetricCard label="Strict Fix Rate" value={metrics.strict_fix_rate ?? 0} />
        <MetricCard label="Raw Fix Rate" value={metrics.raw_fix_rate ?? 0} />
        <MetricCard label="Degraded Runs" value={metrics.degraded_run_count ?? 0} />
        <MetricCard label="Median Patch Size" value={metrics.median_patch_size ?? 0} />
        <MetricCard label="Regression Rate" value={metrics.regression_rate ?? 0} />
      </div>

      <section className="panel-muted p-5">
        <h3 className="text-lg font-semibold">Historical Reports</h3>
        <div className="mt-4 space-y-3">
          {(benchmarks.data?.reports ?? []).map((report) => (
            <div key={report.run_id} className="rounded-xl border border-ink-200/70 px-4 py-3 dark:border-white/10">
              <div className="flex flex-wrap items-center justify-between gap-3">
                <div>
                  <div className="font-medium">{report.run_id}</div>
                  <div className="mt-1 font-mono text-xs text-ink-500 dark:text-ink-300">{report.report_path}</div>
                </div>
                <div className="flex items-center gap-2">
                  <StatusBadge value="strict" />
                  <span className="text-sm">{String(report.metrics.strict_fix_rate ?? 0)}</span>
                  <StatusBadge value="degraded" />
                  <span className="text-sm">{String(report.metrics.degraded_run_count ?? 0)}</span>
                </div>
              </div>
              <div className="mt-3 grid gap-2 md:grid-cols-2 xl:grid-cols-4">
                {Object.entries(report.metrics).slice(0, 6).map(([key, value]) => (
                  <div key={key} className="rounded-lg bg-ink-50 px-3 py-2 text-sm dark:bg-white/5">
                    <div className="font-mono text-xs uppercase tracking-[0.14em] text-ink-500 dark:text-ink-300">{key}</div>
                    <div className="mt-1 font-medium">{String(value)}</div>
                  </div>
                ))}
              </div>
            </div>
          ))}
        </div>
      </section>
    </div>
  );
}
