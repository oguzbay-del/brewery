import { createContext, useCallback, useContext, useEffect, useRef, useState } from 'react';
import type { ReactNode } from 'react';

export type ThemeChoice = 'light' | 'dark' | 'high-contrast';

interface ThemeContextValue {
  theme: ThemeChoice;
  setTheme: (theme: ThemeChoice) => void;
}

const STORAGE_KEY = 'demleme.theme';
const ThemeContext = createContext<ThemeContextValue | null>(null);

function isTheme(value: string | null): value is ThemeChoice {
  return value === 'light' || value === 'dark' || value === 'high-contrast';
}

function readStoredTheme(): ThemeChoice | null {
  try {
    const value = globalThis.localStorage?.getItem(STORAGE_KEY) ?? null;
    return isTheme(value) ? value : null;
  } catch {
    return null;
  }
}

export function systemTheme(): ThemeChoice {
  if (typeof window === 'undefined' || typeof window.matchMedia !== 'function') return 'light';
  if (window.matchMedia('(prefers-contrast: more)').matches) return 'high-contrast';
  return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
}

function applyTheme(theme: ThemeChoice): void {
  if (typeof document === 'undefined') return;
  const dark =
    typeof window !== 'undefined' &&
    typeof window.matchMedia === 'function' &&
    window.matchMedia('(prefers-color-scheme: dark)').matches;
  const cssTheme = theme === 'high-contrast' ? (dark ? 'contrast-dark' : 'contrast') : theme;
  document.documentElement.dataset.theme = cssTheme;
  document.documentElement.dataset.themePreference = theme;
}

export function ThemeProvider({ children }: { children: ReactNode }) {
  const [theme, setThemeState] = useState<ThemeChoice>(() => readStoredTheme() ?? systemTheme());
  const hasSavedPreference = useRef(readStoredTheme() !== null);

  useEffect(() => {
    applyTheme(theme);
  }, [theme]);

  useEffect(() => {
    if (typeof window === 'undefined' || typeof window.matchMedia !== 'function') return;
    const colorScheme = window.matchMedia('(prefers-color-scheme: dark)');
    const contrast = window.matchMedia('(prefers-contrast: more)');
    const update = (): void => {
      if (!hasSavedPreference.current) {
        setThemeState(systemTheme());
      } else if (theme === 'high-contrast') {
        applyTheme(theme);
      }
    };
    colorScheme.addEventListener('change', update);
    contrast.addEventListener('change', update);
    return () => {
      colorScheme.removeEventListener('change', update);
      contrast.removeEventListener('change', update);
    };
  }, [theme]);

  const setTheme = useCallback((next: ThemeChoice) => {
    hasSavedPreference.current = true;
    setThemeState(next);
    try {
      globalThis.localStorage?.setItem(STORAGE_KEY, next);
    } catch {
      // The current session still changes when storage is unavailable.
    }
  }, []);

  return <ThemeContext.Provider value={{ theme, setTheme }}>{children}</ThemeContext.Provider>;
}

export function useTheme(): ThemeContextValue {
  const value = useContext(ThemeContext);
  if (!value) throw new Error('useTheme must be used within ThemeProvider');
  return value;
}
