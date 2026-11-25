import { useEffect, useState } from 'react';
import {
  resolveLocale,
  translate,
  type Locale,
  type TextKey,
  type TextParams,
} from './catalog';

export function useText(key: TextKey | string, params: TextParams = {}): string {
  const [locale, setLocale] = useState<Locale>(() => resolveLocale());

  useEffect(() => {
    const update = (): void => setLocale(resolveLocale());
    window.addEventListener('storage', update);
    return () => window.removeEventListener('storage', update);
  }, []);

  return translate(key, locale, params);
}
