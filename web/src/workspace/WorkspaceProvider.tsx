import { createContext, useContext, useEffect, useState, type ReactNode } from 'react';
import { apiFetch } from '../api/client';

interface WorkspaceSummary {
  id: string;
}
interface SessionInfo {
  workspaces: WorkspaceSummary[];
}

export type WorkspaceState =
  | { status: 'loading'; workspaceId: null }
  | { status: 'ready'; workspaceId: string }
  | { status: 'empty'; workspaceId: null }
  | { status: 'error'; workspaceId: null; error: unknown };

const STORAGE_KEY = 'gristora.workspace_id';
const WorkspaceContext = createContext<WorkspaceState | null>(null);

function savedWorkspaceId(): string | null {
  try {
    return sessionStorage.getItem(STORAGE_KEY);
  } catch {
    return null;
  }
}

function saveWorkspaceId(value: string): void {
  try {
    sessionStorage.setItem(STORAGE_KEY, value);
  } catch {
    // Storage can be unavailable in private browsing.
  }
}

export function WorkspaceProvider({ children }: { children: ReactNode }) {
  const [state, setState] = useState<WorkspaceState>({ status: 'loading', workspaceId: null });

  useEffect(() => {
    const controller = new AbortController();
    apiFetch<SessionInfo>('/v1/me/session', { signal: controller.signal }).then(
      (session) => {
        if (controller.signal.aborted) return;
        const workspaces = session.workspaces ?? [];
        const remembered = savedWorkspaceId();
        const selected = workspaces.find((workspace) => workspace.id === remembered) ?? workspaces[0];
        if (!selected) {
          setState({ status: 'empty', workspaceId: null });
          return;
        }
        saveWorkspaceId(selected.id);
        setState({ status: 'ready', workspaceId: selected.id });
      },
      (error: unknown) => {
        if (!controller.signal.aborted) setState({ status: 'error', workspaceId: null, error });
      },
    );
    return () => controller.abort();
  }, []);

  return <WorkspaceContext.Provider value={state}>{children}</WorkspaceContext.Provider>;
}

export function useWorkspace(): WorkspaceState {
  const state = useContext(WorkspaceContext);
  if (!state) throw new Error('useWorkspace requires WorkspaceProvider');
  return state;
}
