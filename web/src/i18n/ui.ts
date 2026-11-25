import type { Locale } from './catalog';

const uiTexts = {
  tr: {
    appName: 'Demleme',
    skipToContent: 'Ana içeriğe geç',
    navigation: 'Ana gezinme',
    home: 'Ana sayfa',
    recipes: 'Tarifler',
    batches: 'Partiler',
    settings: 'Ayarlar',
    theme: 'Tema',
    light: 'Açık',
    dark: 'Koyu',
    highContrast: 'Yüksek kontrast',
  },
  en: {
    appName: 'Demleme',
    skipToContent: 'Skip to content',
    navigation: 'Main navigation',
    home: 'Home',
    recipes: 'Recipes',
    batches: 'Batches',
    settings: 'Settings',
    theme: 'Theme',
    light: 'Light',
    dark: 'Dark',
    highContrast: 'High contrast',
  },
} satisfies Record<Locale, Record<string, string>>;

export function getUiTextKeys(locale: Locale): string[] {
  return Object.keys(uiTexts[locale]).sort();
}

export function translateUi(key: string, locale: Locale): string | undefined {
  if (!key.startsWith('ui.')) return undefined;
  return uiTexts[locale][key.slice(3)];
}
