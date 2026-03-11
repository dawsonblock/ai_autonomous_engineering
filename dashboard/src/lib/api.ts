export type WorkflowSummary = {
  workflow_id: string;
  workflow_type: string;
  status: string;
  started_at?: string | null;
  updated_at?: string | null;
  completed_at?: string | null;
  metadata: Record<string, unknown>;
  final_states: Record<string, string>;
  event_count: number;
  active_tasks: string[];
  trust_levels: string[];
};

export type WorkflowDetail = {
  summary: WorkflowSummary;
  launch_request: Record<string, unknown>;
  events: Array<Record<string, unknown>>;
  memory_snapshot: Record<string, unknown>;
  artifacts: Record<string, unknown>;
  planner: Record<string, unknown>;
};

export type Diagnostic = {
  name: string;
  status: string;
  summary: string;
  details: Record<string, unknown>;
};

export type BenchmarkSummary = {
  run_id: string;
  metrics: Record<string, number>;
  report_path: string;
  markdown_report_path: string;
  generated_at?: string;
};

export type RuntimeOverrideProfile = {
  controller_concurrency?: number | null;
  planner: Record<string, unknown>;
  localization: Record<string, number>;
  ui: Record<string, unknown>;
  updated_at?: string;
};

const API_ROOT = "/api";

async function fetchJson<T>(path: string, init?: RequestInit): Promise<T> {
  const response = await fetch(`${API_ROOT}${path}`, {
    headers: {
      "Content-Type": "application/json",
      ...(init?.headers ?? {}),
    },
    ...init,
  });
  if (!response.ok) {
    const payload = await response.text();
    throw new Error(payload || `Request failed: ${response.status}`);
  }
  return response.json() as Promise<T>;
}

export const api = {
  listWorkflows: () => fetchJson<WorkflowSummary[]>("/workflows"),
  getWorkflow: (workflowId: string) => fetchJson<WorkflowDetail>(`/workflows/${workflowId}`),
  launchWorkflow: (payload: Record<string, unknown>) =>
    fetchJson<WorkflowDetail>("/workflows", { method: "POST", body: JSON.stringify(payload) }),
  cancelWorkflow: (workflowId: string) =>
    fetchJson<{ workflow_id: string; cancelled: boolean }>(`/workflows/${workflowId}/cancel`, {
      method: "POST",
    }),
  rerunWorkflow: (workflowId: string) =>
    fetchJson<WorkflowDetail>(`/workflows/${workflowId}/rerun`, { method: "POST" }),
  benchmarkSummary: () => fetchJson<{ latest: BenchmarkSummary | null; reports: BenchmarkSummary[] }>("/benchmarks"),
  runBenchmarks: () => fetchJson<BenchmarkSummary>("/benchmarks/run", { method: "POST", body: JSON.stringify({}) }),
  settings: () => fetchJson<RuntimeOverrideProfile>("/settings/runtime-overrides"),
  updateSettings: (payload: RuntimeOverrideProfile) =>
    fetchJson<RuntimeOverrideProfile>("/settings/runtime-overrides", {
      method: "PUT",
      body: JSON.stringify(payload),
    }),
  diagnostics: () => fetchJson<Diagnostic[]>("/health"),
  workflowArtifacts: (workflowId: string) => fetchJson<Record<string, unknown>>(`/artifacts/workflows/${workflowId}`),
};
