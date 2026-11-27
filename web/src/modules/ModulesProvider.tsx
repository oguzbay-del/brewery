import { createContext, useContext, useEffect, useMemo, useState, type ReactNode } from 'react';
import { onModuleError } from '../api/client';
import type { ModuleErrorKind } from '../api/errors';
import type { WorkspaceModule } from '../api/modules';
import { useWorkspace } from '../workspace/useWorkspace';
import { useModules } from './useModules';

/**
 * `unknown` means the state could not be loaded (or there is no provider). The server still
 * enforces the module state, so the interface hides nothing for `unknown`.
 */
export type ModuleAccess = 'on' | 'read_only' | 'off' | 'loading' | 'unknown';

interface ModulesContextValue {
  loading: boolean;
  modules: WorkspaceModule[] | null;
  notice: ModuleErrorKind | null;
  dismissNotice: () => void;
}

const ModulesContext = createContext<ModulesContextValue | null>(null);

export function ModulesProvider({ children }: { children: ReactNode }) {
  const workspace = useWorkspace();
  const result = useModules(workspace.workspaceId);
  const [notice, setNotice] = useState<ModuleErrorKind | null>(null);

  useEffect(() => onModuleError(setNotice), []);

  const value = useMemo<ModulesContextValue>(
    () => ({
      loading: workspace.status === 'loading' || result.status === 'loading',
      modules: result.status === 'ready' ? result.modules : null,
      notice,
      dismissNotice: () => setNotice(null),
    }),
    [workspace.status, result, notice],
  );

  return <ModulesContext.Provider value={value}>{children}</ModulesContext.Provider>;
}

/** The last module problem the API returned for a write (403), and a way to clear it. */
export function useModuleNotice(): { notice: ModuleErrorKind | null; dismiss: () => void } {
  const value = useContext(ModulesContext);
  return { notice: value?.notice ?? null, dismiss: value?.dismissNotice ?? (() => undefined) };
}

/** Access state of one module id. `null` means the screen needs no module. */
export function useModuleAccess(moduleId: string | null): ModuleAccess {
  const value = useContext(ModulesContext);
  if (moduleId === null) return 'on';
  if (!value) return 'unknown';
  if (!value.modules) return value.loading ? 'loading' : 'unknown';
  const module = value.modules.find((candidate) => candidate.id === moduleId);
  // A module the API does not list is treated as off.
  if (!module || !module.enabled) return 'off';
  return module.read_only ? 'read_only' : 'on';
}
