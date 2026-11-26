import { apiFetch } from './client';

export interface WorkspaceModule {
  id: string;
  enabled: boolean;
  read_only: boolean;
}

export interface WorkspaceModulesResponse {
  modules: WorkspaceModule[];
}

/** Loads the module states for a workspace from the API. */
export function fetchWorkspaceModules(
  workspaceId: string,
  signal?: AbortSignal,
): Promise<WorkspaceModulesResponse> {
  return apiFetch<WorkspaceModulesResponse>(
    `/v1/meta?workspace_id=${encodeURIComponent(workspaceId)}`,
    { signal },
  );
}
