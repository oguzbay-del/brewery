import { act, renderHook } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { LOCALE_STORAGE_KEY, getTextKeys, resolveLocale, translate } from './catalog';
import { useText } from './useText';

afterEach(() => {
  window.localStorage.clear();
  vi.restoreAllMocks();
});

describe('text catalog', () => {
  it('has exact key parity between Turkish and English', () => {
    expect(getTextKeys('tr')).toEqual(getTextKeys('en'));
  });

  it('replaces named placeholders and leaves unspecified placeholders intact', () => {
    expect(
      translate('errors.out_of_range', 'en', {
        label: 'Original gravity',
        min: 1,
        max: 1.12,
        unit: 'SG',
      }),
    ).toBe('Original gravity must be between 1 and 1.12 SG.');
    expect(translate('errors.required', 'tr')).toContain('{label}');
  });

  it('returns the missing key and warns once for that key', () => {
    const warning = vi.spyOn(console, 'warn').mockImplementation(() => undefined);
    expect(translate('missing.example', 'en')).toBe('missing.example');
    expect(translate('missing.example', 'en')).toBe('missing.example');
    expect(warning).toHaveBeenCalledTimes(1);
  });

  it('prefers local storage, then the browser language, then English', () => {
    expect(resolveLocale({ getItem: () => 'tr' }, 'en-US')).toBe('tr');
    expect(resolveLocale({ getItem: () => null }, 'tr-TR')).toBe('tr');
    expect(resolveLocale({ getItem: () => null }, 'fr-FR')).toBe('en');
    expect(resolveLocale({ getItem: () => 'invalid' }, undefined)).toBe('en');
  });

  it('uses the active locale in the hook and responds to storage events', () => {
    window.localStorage.setItem(LOCALE_STORAGE_KEY, 'tr');
    const { result } = renderHook(() => useText('errors.required', { label: 'OG' }));
    expect(result.current).toBe('OG gerekli.');

    act(() => {
      window.localStorage.setItem(LOCALE_STORAGE_KEY, 'en');
      window.dispatchEvent(new StorageEvent('storage', { key: LOCALE_STORAGE_KEY }));
    });
    expect(result.current).toBe('OG is required.');
  });
});
