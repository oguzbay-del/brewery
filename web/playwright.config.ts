import { defineConfig } from '@playwright/test';

const port = 4173;

export default defineConfig({
  testDir: './e2e',
  reporter: 'list',
  use: {
    baseURL: `http://127.0.0.1:${port}`,
    launchOptions: {
      executablePath: process.env['PW_CHROMIUM_PATH'] ?? '/opt/pw-browsers/chromium',
    },
  },
  webServer: {
    command: `npm run build && npm run preview -- --host 127.0.0.1 --port ${port} --strictPort`,
    url: `http://127.0.0.1:${port}`,
    reuseExistingServer: !process.env['CI'],
    timeout: 120_000,
  },
});
