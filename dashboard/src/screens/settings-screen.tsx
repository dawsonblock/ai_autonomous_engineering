import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useEffect, useState } from "react";

import { StatusBadge } from "@/components/status-badge";
import { api, type RuntimeOverrideProfile } from "@/lib/api";

export function SettingsScreen() {
  const queryClient = useQueryClient();
  const settings = useQuery({ queryKey: ["settings"], queryFn: api.settings });
  const diagnostics = useQuery({ queryKey: ["diagnostics"], queryFn: api.diagnostics, refetchInterval: 10000 });
  const [draft, setDraft] = useState<RuntimeOverrideProfile | null>(null);
  const [localizationText, setLocalizationText] = useState("{}");
  const mutation = useMutation({
    mutationFn: (payload: RuntimeOverrideProfile) => api.updateSettings(payload),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ["settings"] });
    },
  });

  const value = draft ?? settings.data ?? { planner: {}, localization: {}, ui: {} };

  useEffect(() => {
    if (!draft && settings.data) {
      setLocalizationText(JSON.stringify(settings.data.localization ?? {}, null, 2));
    }
  }, [draft, settings.data]);

  const save = () => {
    try {
      const localization = JSON.parse(localizationText || "{}") as Record<string, number>;
      mutation.mutate({ ...value, localization });
    } catch {
      return;
    }
  };

  return (
    <div className="grid gap-4 xl:grid-cols-[1fr_0.95fr]">
      <section className="panel-muted space-y-5 p-5">
        <div>
          <h2 className="text-xl font-semibold">Runtime Overrides</h2>
          <p className="mt-2 text-sm text-ink-600 dark:text-ink-200">
            Overrides are written to <code>.artifacts/dashboard/runtime_overrides.json</code> and never mutate tracked config.
          </p>
        </div>
        <label className="block space-y-2">
          <span className="text-sm font-medium">Controller Concurrency</span>
          <input
            type="number"
            className="w-full rounded-xl border border-ink-200 bg-white px-3 py-2 dark:border-white/10 dark:bg-white/5"
            value={value.controller_concurrency ?? ""}
            onChange={(event) =>
              setDraft({
                ...value,
                controller_concurrency: event.target.value ? Number(event.target.value) : null,
              })
            }
          />
        </label>
        <label className="block space-y-2">
          <span className="text-sm font-medium">Localization Weights (JSON)</span>
          <textarea
            className="min-h-32 w-full rounded-xl border border-ink-200 bg-white px-3 py-2 font-mono text-sm dark:border-white/10 dark:bg-white/5"
            value={draft ? localizationText : JSON.stringify(value.localization ?? {}, null, 2)}
            onFocus={() => setLocalizationText(JSON.stringify(value.localization ?? {}, null, 2))}
            onChange={(event) => setLocalizationText(event.target.value)}
          />
        </label>
        <button
          type="button"
          onClick={save}
          className="rounded-xl bg-ink-900 px-4 py-2.5 text-sm font-semibold text-white dark:bg-white dark:text-ink-900"
        >
          Save overrides
        </button>
      </section>

      <section className="panel-muted p-5">
        <h3 className="text-lg font-semibold">Diagnostics</h3>
        <div className="mt-4 space-y-3">
          {(diagnostics.data ?? []).map((diagnostic) => (
            <div key={diagnostic.name} className="rounded-xl border border-ink-200/70 px-4 py-3 dark:border-white/10">
              <div className="flex items-center justify-between gap-3">
                <div className="font-medium">{diagnostic.name}</div>
                <StatusBadge value={diagnostic.status} />
              </div>
              <div className="mt-2 text-sm text-ink-600 dark:text-ink-200">{diagnostic.summary}</div>
            </div>
          ))}
        </div>
      </section>
    </div>
  );
}
