import { useSyncExternalStore } from 'react';
import type { ShellLabels } from './AppShell';
import { ModulesList } from '../settings/ModulesList';

import { ContainersScreen } from '../containers/ContainersScreen';
import { ModuleGate, ModuleNotice } from '../modules/ModuleGate';
import { ROUTE_MODULES, ROUTES, type AppRoute } from './navigation';

export { ROUTES, type AppRoute };

export function readHashRoute(hash: string): AppRoute {
  const candidate = hash.startsWith('#') ? hash.slice(1) : hash;
  const route = candidate || '/';
  return ROUTES.includes(route as AppRoute) ? (route as AppRoute) : '/';
}

function subscribe(onChange: () => void): () => void {
  window.addEventListener('hashchange', onChange);
  return () => window.removeEventListener('hashchange', onChange);
}

function getSnapshot(): AppRoute {
  return readHashRoute(window.location.hash);
}

export function useHashRoute(): AppRoute {
  return useSyncExternalStore(subscribe, getSnapshot, () => '/');
}

export function RouteContent({ labels }: { labels: ShellLabels }) {
  const route = useHashRoute();
  const title: Record<AppRoute, string> = {
    '/': labels.home,
    '/recipes': labels.recipes,
    '/batches': labels.batches,
    '/containers': labels.containers,
    '/settings': labels.settings,
  };

  return (
    <section aria-labelledby="app-route-heading">
      <h1 id="app-route-heading">{title[route]}</h1>
      <ModuleNotice />
      <ModuleGate moduleId={ROUTE_MODULES[route]}>
        {route === '/containers' && <ContainersScreen />}
        {route === '/settings' && <ModulesList />}
      </ModuleGate>
    </section>
  );
}
