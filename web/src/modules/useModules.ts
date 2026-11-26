import { useEffect, useState } from 'react';
import { fetchWorkspaceModules, type WorkspaceModule } from '../api/modules';

export type ModulesState =
  | { status: 'idle' }
  | { status: 'loading' }
  | { status: 'ready'; modules: WorkspaceModule[] }
  | { status: 'error'; error: unknown };

export function useModules(workspaceId: string | null): ModulesState {
  const [result, setResult] = useState<{ workspaceId: string; state: ModulesState } | null>(null);

  useEffect(() => {
    if (!workspaceId) return;
    const controller = new AbortController();
    fetchWorkspaceModules(workspaceId, controller.signal).then(
      ({ modules }) => {
        if (!controller.signal.aborted) {
          setResult({ workspaceId, state: { status: 'ready', modules } });
        }
      },
      (error: unknown) => {
        if (!controller.signal.aborted) {
          setResult({ workspaceId, state: { status: 'error', error } });
        }
      },
    );
    return () => controller.abort();
  }, [workspaceId]);

  if (!workspaceId) return { status: 'idle' };
  if (result?.workspaceId !== workspaceId) return { status: 'loading' };
  return result.state;
}
