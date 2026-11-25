import { ESLint } from 'eslint';
import { describe, expect, it } from 'vitest';

const webRoot = new URL('..', import.meta.url).pathname;
const configPath = new URL('../eslint.config.js', import.meta.url).pathname;
const probePath = new URL('./__boundary_probe__.ts', import.meta.url).pathname;
const eslint = new ESLint({ cwd: webRoot, overrideConfigFile: configPath });

async function restrictedImportMessages(source: string) {
  const results = await eslint.lintText(source, { filePath: probePath });
  return results.flatMap((result) =>
    result.messages.filter((message) => message.ruleId === 'no-restricted-imports'),
  );
}

describe('client import boundary', () => {
  it.each([
    ['server source tree', "import server from '../../sunucu/crates/api/src/lib';"],
    ['server package path', "import server from 'sunucu/crates/api';"],
    ['engine package', "import engine from 'engine';"],
    [
      'scoped engine module re-export',
      "export { default } from '@gristora/engine-service/client';",
    ],
  ])('rejects %s', async (_label, source) => {
    const messages = await restrictedImportMessages(source);
    expect(messages.length).toBeGreaterThan(0);
  });

  it('allows normal dependencies and local modules', async () => {
    const messages = await restrictedImportMessages(
      "import React from 'react';\nimport App from './App';",
    );
    expect(messages).toHaveLength(0);
  });
});
