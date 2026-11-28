import { defineConfig } from '@playwright/test';

const port = 4174;

export default defineConfig({
  testDir: '.',
  testMatch: 'gallery.spec.ts',
  reporter: 'list',
  use: {
    baseURL: `http://127.0.0.1:${port}`,
    viewport: { width: 1366, height: 1024 },
    deviceScaleFactor: 1,
    reducedMotion: 'reduce',
  },
  webServer: {
    command: `npm run build:kit && npm run preview -- --mode kit --host 127.0.0.1 --port ${port} --strictPort`,
    url: `http://127.0.0.1:${port}/kit.html`,
    cwd: new URL('../../../', import.meta.url).pathname,
    reuseExistingServer: false,
    timeout: 120_000,
  },
});
