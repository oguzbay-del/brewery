import type { ReactNode } from 'react';
import { BrandSymbol } from './BrandSymbol';
import { useModuleAccess } from '../modules/ModulesProvider';
import { ROUTE_MODULES, type AppRoute } from './navigation';
import { useTheme } from './ThemeProvider';
import type { ThemeChoice } from './ThemeProvider';

export interface ShellLabels {
  appName: string;
  skipToContent: string;
  navigation: string;
  home: string;
  recipes: string;
  batches: string;
  containers: string;
  settings: string;
  readOnly: string;
  theme: string;
  light: string;
  dark: string;
  highContrast: string;
}

interface AppShellProps {
  labels: ShellLabels;
  children: ReactNode;
}

function NavLink({
  route,
  label,
  readOnlyLabel,
}: {
  route: AppRoute;
  label: string;
  readOnlyLabel: string;
}) {
  const access = useModuleAccess(ROUTE_MODULES[route]);
  // Hide the entry of a module that is off; keep a read-only entry with text.
  if (access === 'off') return null;
  return (
    <a href={`#${route}`}>
      {label}
      {access === 'read_only' && <span className="app-shell__nav-mark"> ({readOnlyLabel})</span>}
    </a>
  );
}

export function AppShell({ labels, children }: AppShellProps) {
  const { theme, setTheme } = useTheme();

  return (
    <div className="app-shell">
      <a className="app-shell__skip-link" href="#main-content">
        {labels.skipToContent}
      </a>
      <header className="app-shell__header">
        <a className="app-shell__brand" href="#/" aria-label={labels.appName}>
          <BrandSymbol />
          <span className="app-shell__wordmark">{labels.appName}</span>
        </a>
        <nav className="app-shell__navigation" aria-label={labels.navigation}>
          <NavLink route="/" label={labels.home} readOnlyLabel={labels.readOnly} />
          <NavLink route="/recipes" label={labels.recipes} readOnlyLabel={labels.readOnly} />
          <NavLink route="/batches" label={labels.batches} readOnlyLabel={labels.readOnly} />
          <NavLink route="/containers" label={labels.containers} readOnlyLabel={labels.readOnly} />
          <NavLink route="/settings" label={labels.settings} readOnlyLabel={labels.readOnly} />
        </nav>
        <label className="app-shell__theme-control">
          <span>{labels.theme}</span>
          <select
            aria-label={labels.theme}
            value={theme}
            onChange={(event) => setTheme(event.currentTarget.value as ThemeChoice)}
          >
            <option value="light">{labels.light}</option>
            <option value="dark">{labels.dark}</option>
            <option value="high-contrast">{labels.highContrast}</option>
          </select>
        </label>
      </header>
      <main className="app-shell__main" id="main-content" tabIndex={-1}>
        {children}
      </main>
    </div>
  );
}
