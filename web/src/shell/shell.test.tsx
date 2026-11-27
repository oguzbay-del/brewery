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
  readOnly: 'Read only',
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

function renderShell() {
  return render(
    <ThemeProvider>
      <AppShell labels={labels}>
        <h1>Page</h1>
      </AppShell>
    </ThemeProvider>,
  );
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
  it('provides a skip link and main landmark', () => {
    renderShell();
    const skip = screen.getByRole('link', { name: 'Skip to content' });
    expect(skip).toHaveAttribute('href', '#main-content');
    expect(screen.getByRole('main')).toHaveAttribute('id', 'main-content');
  });

  it('supports keyboard theme selection and persistence', async () => {
    renderShell();
    const control = screen.getByRole('combobox', { name: 'Theme' });
    control.focus();
    expect(control).toHaveFocus();
    fireEvent.change(control, { target: { value: 'dark' } });
    await waitFor(() => expect(document.documentElement.dataset.theme).toBe('dark'));
    expect(window.localStorage.getItem('demleme.theme')).toBe('dark');
  });

  it('maps high contrast to the existing token set', async () => {
    renderShell();
    fireEvent.change(screen.getByRole('combobox', { name: 'Theme' }), {
      target: { value: 'high-contrast' },
    });
    await waitFor(() => expect(document.documentElement.dataset.theme).toBe('contrast'));
    expect(document.documentElement.dataset.themePreference).toBe('high-contrast');
  });

  it('follows system contrast and colour scheme', () => {
    Object.defineProperty(window, 'matchMedia', {
      configurable: true,
      value: media({
        '(prefers-color-scheme: dark)': true,
        '(prefers-contrast: more)': true,
      }),
    });
    expect(systemTheme()).toBe('high-contrast');
  });

  it('accepts only supported hash routes', () => {
    expect(readHashRoute('#/recipes')).toBe('/recipes');
    expect(readHashRoute('#/batches')).toBe('/batches');
    expect(readHashRoute('#/settings')).toBe('/settings');
    expect(readHashRoute('#/unknown')).toBe('/');
  });
});
