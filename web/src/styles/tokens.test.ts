import tokensJson from '../../../tasarim-sistemi/tokens.json?raw';
import { describe, expect, it } from 'vitest';

type Token = { name: string; value: string };
type Tokens = Record<string, { tokens: Token[] }>;

const tokens = JSON.parse(tokensJson) as Tokens;

const find = (group: string, name: string): string => {
  const token = tokens[group]?.tokens.find((t) => t.name === name);
  if (!token) throw new Error(`token ${name} missing in group ${group}`);
  return token.value;
};
const px = (group: string, name: string): number => Number.parseFloat(find(group, name));

describe('structural tokens', () => {
  it('bp-sidebar is sidebar-ideal plus content-min', () => {
    expect(px('layout', 'bp-sidebar')).toBe(
      px('layout', 'sidebar-ideal') + px('layout', 'content-min'),
    );
  });

  it('bp-inspector adds inspector-ideal to bp-sidebar', () => {
    expect(px('layout', 'bp-inspector')).toBe(
      px('layout', 'sidebar-ideal') + px('layout', 'content-min') + px('layout', 'inspector-ideal'),
    );
  });

  it('z values increase strictly in the listed order', () => {
    const order = [
      'z-base',
      'z-sticky',
      'z-sidebar-overlay',
      'z-popover',
      'z-sheet',
      'z-dialog',
      'z-toast',
      'z-skip-link',
    ];
    const values = order.map((name) => Number(find('z', name)));
    for (let i = 1; i < values.length; i += 1) {
      expect(values[i]).toBeGreaterThan(values[i - 1] ?? Infinity);
    }
  });
});
