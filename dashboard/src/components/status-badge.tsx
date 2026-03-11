type Props = {
  value: string;
};

const palette: Record<string, string> = {
  strict: "bg-signal-green/10 text-signal-green ring-signal-green/20",
  degraded: "bg-signal-amber/10 text-signal-amber ring-signal-amber/20",
  docker: "bg-signal-blue/10 text-signal-blue ring-signal-blue/20",
  local: "bg-ink-400/10 text-ink-700 ring-ink-400/20 dark:text-ink-100",
  running: "bg-signal-blue/10 text-signal-blue ring-signal-blue/20",
  completed: "bg-signal-green/10 text-signal-green ring-signal-green/20",
  cancelled: "bg-signal-amber/10 text-signal-amber ring-signal-amber/20",
  failed: "bg-signal-red/10 text-signal-red ring-signal-red/20",
};

export function StatusBadge({ value }: Props) {
  const key = value.toLowerCase();
  return (
    <span
      className={`inline-flex items-center rounded-full px-2.5 py-1 text-xs font-semibold uppercase tracking-[0.16em] ring-1 ${
        palette[key] ?? "bg-ink-200/60 text-ink-700 ring-ink-300/50 dark:bg-white/10 dark:text-ink-50 dark:ring-white/10"
      }`}
    >
      {value.replaceAll("_", " ")}
    </span>
  );
}
