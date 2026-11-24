import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { AppShell, type ShellLabels } from './AppShell';
import { ThemeProvider, systemTheme } from './ThemeProvider';
import { readHashRoute } from './routes';

const labels: ShellLabels = {
  appName: 'Demleme',
  skipToContent: 'Skip to content',
  navigation: 'Primary navigation',
  home: 'Home',
  recipes: 'Recipes',
  batches: 'Batches',
  settings: 'Settings',
  theme: 'Theme',
  light: 'Light',
  dark: 'Dark',
  highContrast: 'High contrast',
};

function media(matches: Record<string, boolean>) {
  return (query: string) =>
    ({
      matches: matches[query] ?? false,
      media: query,
      onchange: null,
      addEventListener: vi.fn(),
      removeEventListener: vi.fn(),
      addListener: vi.fn(),
      removeListener: vi.fn(),
      dispatchEvent: vi.fn(),
    }) as unknown as MediaQueryList;
}

beforeEach(() => {
  window.localStorage.clear();
  Object.defineProperty(window, 'matchMedia', {
    configurable: true,
    value: media({
      '(prefers-color-scheme: dark)': false,
      '(prefers-contrast: more)': false,
    }),
  });
});

describe('app shell', () => {
  it('provides a skip link and a main landmark', () => {
    render(
      <ThemeProvider>
        <AppShell labels={labels}>
          <h1>Page</h1>
        </AppShell>
      </ThemeProvider>,
    );
    expect(
      screen.getByRole('link', { name: 'Skip to content' }),
    ).toHaveAttribute('href', '#main-content');
    expect(screen.getByRole('main')).toHaveAttribute('id', 'main-content');
  });

  it('keeps the theme control keyboard reachable and applies the selected theme', async () => {
    render(
      <ThemeProvider>
        <AppShell labels={labels}>
          <h1>Page</h1>
        </AppShell>
      </ThemeProvider>,
    );
    const control = screen.getByRole('combobox', { name: 'Theme' });
    control.focus();
    expect(control).toHaveFocus();
    fireEvent.change(control, { target: { value: 'dark' } });
    await waitFor(() => expect(document.documentElement.dataset.theme).toBe('dark'));
    expect(window.localStorage.getItem('demleme.theme')).toBe('dark');
  });

  it('maps high contrast to the existing light and dark token sets', async () => {
    render(
      <ThemeProvider>
        <AppShell labels={labels}>
          <h1>Page</h1>
        </AppShell>
      </ThemeProvider>,
    );
    fireEvent.change(screen.getByRole('combobox', { name: 'Theme' }), {
      target: { value: 'high-contrast' },
    });
    await waitFor(() => expect(document.documentElement.dataset.theme).toBe('contrast'));
    expect(document.documentElement.dataset.themePreference).toBe('high-contrast');
  });

  it('follows system contrast and colour scheme when no preference is saved', () => {
    Object.defineProperty(window, 'matchMedia', {
      configurable: true,
      value: media({
        '(prefers-color-scheme: dark)': true,
        '(prefers-contrast: more)': true,
      }),
    });
    expect(systemTheme()).toBe('high-contrast');
  });

  it('accepts only the four supported hash routes', () => {
    expect(readHashRoute('#/recipes')).toBe('/recipes');
    expect(readHashRoute('#/batches')).toBe('/batches');
    expect(readHashRoute('#/settings')).toBe('/settings');
    expect(readHashRoute('#/unknown')).toBe('/');
  });
});
