import { useCallback, useEffect, useState } from 'react';
import { resolveLocale, translate, type Locale, type TextKey, type TextParams } from './catalog';

function useLocale(): Locale {
  const [locale, setLocale] = useState<Locale>(() => resolveLocale());

  useEffect(() => {
    const update = (): void => setLocale(resolveLocale());
    window.addEventListener('storage', update);
    return () => window.removeEventListener('storage', update);
  }, []);

  return locale;
}

export function useText(key: TextKey, params: TextParams = {}): string {
  return translate(key, useLocale(), params);
}

/** Returns a translate function for keys that are built at run time (for example a status). */
export function useTranslate(): (key: string, params?: TextParams) => string {
  const locale = useLocale();
  return useCallback((key, params = {}) => translate(key, locale, params), [locale]);
}
