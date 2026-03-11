import type { ReactNode } from "react";

type Props = {
  label: string;
  value: ReactNode;
  hint?: string;
};

export function MetricCard({ label, value, hint }: Props) {
  return (
    <div className="panel p-5">
      <div className="text-xs font-semibold uppercase tracking-[0.18em] text-ink-500 dark:text-ink-300">{label}</div>
      <div className="mt-3 text-3xl font-semibold text-ink-900 dark:text-ink-50">{value}</div>
      {hint ? <div className="mt-2 text-sm text-ink-600 dark:text-ink-200">{hint}</div> : null}
    </div>
  );
}
