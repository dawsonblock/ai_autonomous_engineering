import { createRootRoute, createRoute, createRouter } from "@tanstack/react-router";

import { DashboardLayout } from "@/components/layout";
import { BenchmarksScreen } from "@/screens/benchmarks-screen";
import { LaunchWorkflowScreen } from "@/screens/launch-workflow-screen";
import { OverviewScreen } from "@/screens/overview-screen";
import { SettingsScreen } from "@/screens/settings-screen";
import { WorkflowDetailScreen } from "@/screens/workflow-detail-screen";

const rootRoute = createRootRoute({
  component: DashboardLayout,
});

const overviewRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/",
  component: OverviewScreen,
});

const launchRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/launch",
  component: LaunchWorkflowScreen,
});

const benchmarksRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/benchmarks",
  component: BenchmarksScreen,
});

const settingsRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/settings",
  component: SettingsScreen,
});

const workflowDetailRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/workflows/$workflowId",
  component: WorkflowDetailScreen,
});

const routeTree = rootRoute.addChildren([
  overviewRoute,
  launchRoute,
  benchmarksRoute,
  settingsRoute,
  workflowDetailRoute,
]);

export const router = createRouter({
  routeTree,
  defaultPreload: "intent",
});

declare module "@tanstack/react-router" {
  interface Register {
    router: typeof router;
  }
}
