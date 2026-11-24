import { useEffect, useState } from 'react';
import source from '../../../sozlesme/katalog/katalog.json?raw';

export type Locale = 'tr' | 'en';
export type TextKey =
  | `quantities.${string}`
  | `units.${string}.${'name' | 'symbol'}`
  | `fields.${string}.${'label' | 'hint'}`
  | `errors.${string}`;
export type TextParams = Record<string, string | number>;

interface LocaleCatalog {
  quantities: Record<string, string>;
  units: Record<string, { name: string; symbol: string }>;
  fields: Record<string, { label: string; hint?: string }>;
  errors: Record<string, string>;
}

interface CatalogFile {
  texts: Record<Locale, LocaleCatalog>;
}

const catalog = JSON.parse(source) as CatalogFile;
const warnedKeys = new Set<string>();
export const LOCALE_STORAGE_KEY = 'demleme.locale';

function isLocale(value: string | null | undefined): value is Locale {
  return value === 'tr' || value === 'en';
}

/** Resolves an explicit preference first, then the browser language, then English. */
export function resolveLocale(
  storage?: Pick<Storage, 'getItem'>,
  browserLanguage?: string,
): Locale {
  try {
    const store = storage ?? globalThis.localStorage;
    const saved = store?.getItem(LOCALE_STORAGE_KEY);
    if (isLocale(saved)) return saved;
  } catch {
    // Storage can be disabled by browser privacy settings.
  }

  let language = browserLanguage;
  if (language === undefined) {
    try {
      language = globalThis.navigator?.language;
    } catch {
      language = undefined;
    }
  }
  return language?.toLowerCase().startsWith('tr') ? 'tr' : 'en';
}

function lookup(locale: Locale, key: string): string | undefined {
  const texts = catalog.texts[locale];

  if (key.startsWith('quantities.')) {
    return texts.quantities[key.slice('quantities.'.length)];
  }
  if (key.startsWith('errors.')) {
    return texts.errors[key.slice('errors.'.length)];
  }
  if (key.startsWith('units.')) {
    const [, unit, property] = key.split('.');
    if (unit && (property === 'name' || property === 'symbol')) {
      return texts.units[unit]?.[property];
    }
  }
  if (key.startsWith('fields.')) {
    const rest = key.slice('fields.'.length);
    const field = Object.keys(texts.fields).find((candidate) =>
      rest.startsWith(`${candidate}.`),
    );
    if (field) {
      const property = rest.slice(field.length + 1);
      if (property === 'label' || property === 'hint') return texts.fields[field]?.[property];
    }
  }
  return undefined;
}

/** Lists every string leaf; used to enforce exact TR/EN key parity. */
export function getTextKeys(locale: Locale): string[] {
  const keys: string[] = [];
  const visit = (value: unknown, prefix: string): void => {
    if (typeof value === 'string') {
      keys.push(prefix);
      return;
    }
    if (typeof value !== 'object' || value === null || Array.isArray(value)) return;
    for (const [key, child] of Object.entries(value)) {
      visit(child, prefix ? `${prefix}.${key}` : key);
    }
  };
  visit(catalog.texts[locale], '');
  return keys.sort();
}

/** Resolves a catalog key and replaces the supplied named placeholders. */
export function translate(key: string, locale: Locale, params: TextParams = {}): string {
  const value = lookup(locale, key);
  if (value === undefined) {
    if (!warnedKeys.has(key)) {
      warnedKeys.add(key);
      console.warn(`Missing text catalog key: ${key}`);
    }
    return key;
  }
  return value.replace(/\{([a-zA-Z0-9_]+)\}/g, (match, name: string) =>
    Object.hasOwn(params, name) ? String(params[name]) : match,
  );
}

/** Returns the requested catalog text in the active browser language. */
export function useText(key: TextKey | string, params: TextParams = {}): string {
  const [locale, setLocale] = useState<Locale>(() => resolveLocale());

  useEffect(() => {
    const update = (): void => setLocale(resolveLocale());
    window.addEventListener('storage', update);
    return () => window.removeEventListener('storage', update);
  }, []);

  return translate(key, locale, params);
}
