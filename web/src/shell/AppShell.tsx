import type { ReactNode } from 'react';
import { useTheme } from './ThemeProvider';
import type { ThemeChoice } from './ThemeProvider';

export interface ShellLabels {
  appName: string;
  skipToContent: string;
  navigation: string;
  home: string;
  recipes: string;
  batches: string;
  settings: string;
  theme: string;
  light: string;
  dark: string;
  highContrast: string;
}

interface AppShellProps {
  labels: ShellLabels;
  children: ReactNode;
}

export function AppShell({ labels, children }: AppShellProps) {
  const { theme, setTheme } = useTheme();
  const hasThemeLabels = Boolean(
    labels.theme && labels.light && labels.dark && labels.highContrast,
  );
  const hasNavigationLabels = Boolean(
    labels.navigation &&
      labels.home &&
      labels.recipes &&
      labels.batches &&
      labels.settings,
  );

  return (
    <div className="app-shell">
      {labels.skipToContent && (
        <a className="app-shell__skip-link" href="#main-content">
          {labels.skipToContent}
        </a>
      )}
      <header className="app-shell__header">
        <a className="app-shell__brand" href="#/" aria-label={labels.appName}>
          {labels.appName}
        </a>
        {hasNavigationLabels && (
          <nav className="app-shell__navigation" aria-label={labels.navigation}>
            <a href="#/">{labels.home}</a>
            <a href="#/recipes">{labels.recipes}</a>
            <a href="#/batches">{labels.batches}</a>
            <a href="#/settings">{labels.settings}</a>
          </nav>
        )}
        {hasThemeLabels && (
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
        )}
      </header>
      <main className="app-shell__main" id="main-content" tabIndex={-1}>
        {children}
      </main>
    </div>
  );
}
