import * as Switch from "@radix-ui/react-switch";
import { Link, Outlet, useRouterState } from "@tanstack/react-router";

import { useUiStore } from "@/state/ui-store";

const nav = [
  { to: "/", label: "Overview" },
  { to: "/launch", label: "Launch Workflow" },
  { to: "/benchmarks", label: "Benchmarks" },
  { to: "/settings", label: "Settings" },
];

export function DashboardLayout() {
  const theme = useUiStore((state) => state.theme);
  const setTheme = useUiStore((state) => state.setTheme);
  const pathname = useRouterState({ select: (state) => state.location.pathname });

  return (
    <div className="min-h-screen px-4 py-5 lg:px-6">
      <div className="mx-auto grid min-h-[calc(100vh-2.5rem)] max-w-[1600px] grid-cols-1 gap-4 lg:grid-cols-[280px_minmax(0,1fr)]">
        <aside className="panel flex flex-col gap-6 p-5">
          <div>
            <div className="text-xs font-semibold uppercase tracking-[0.2em] text-signal-blue">AAE</div>
            <h1 className="mt-2 text-2xl font-semibold tracking-tight">Control Center</h1>
            <p className="mt-2 text-sm text-ink-600 dark:text-ink-200">
              Local operator console for workflows, benchmarks, repair provenance, and sandbox trust.
            </p>
          </div>
          <nav className="space-y-2">
            {nav.map((item) => {
              const active = pathname === item.to;
              return (
                <Link
                  key={item.to}
                  to={item.to}
                  className={`flex items-center justify-between rounded-xl px-4 py-3 text-sm font-medium transition ${
                    active
                      ? "bg-ink-900 text-white dark:bg-white dark:text-ink-900"
                      : "panel-muted text-ink-700 hover:border-signal-blue/30 hover:text-ink-900 dark:text-ink-100 dark:hover:text-white"
                  }`}
                >
                  <span>{item.label}</span>
                  <span className="font-mono text-xs">{item.to === "/" ? "00" : item.to.replace("/", "").slice(0, 2).toUpperCase()}</span>
                </Link>
              );
            })}
          </nav>
          <div className="mt-auto panel-muted flex items-center justify-between px-4 py-3">
            <div>
              <div className="text-xs font-semibold uppercase tracking-[0.18em] text-ink-500 dark:text-ink-300">Theme</div>
              <div className="text-sm text-ink-700 dark:text-ink-100">{theme === "dark" ? "Dark mode" : "Light mode"}</div>
            </div>
            <Switch.Root
              checked={theme === "dark"}
              onCheckedChange={(checked) => setTheme(checked ? "dark" : "light")}
              className="relative h-6 w-11 rounded-full bg-ink-300 transition data-[state=checked]:bg-signal-blue"
            >
              <Switch.Thumb className="block h-5 w-5 translate-x-0.5 rounded-full bg-white shadow transition will-change-transform data-[state=checked]:translate-x-[1.35rem]" />
            </Switch.Root>
          </div>
        </aside>
        <main className="panel overflow-hidden">
          <div className="border-b border-ink-200/80 px-6 py-4 dark:border-white/10">
            <div className="text-sm font-medium uppercase tracking-[0.16em] text-ink-500 dark:text-ink-300">Embedded Runtime</div>
          </div>
          <div className="p-6">
            <Outlet />
          </div>
        </main>
      </div>
    </div>
  );
}
