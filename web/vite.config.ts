import react from '@vitejs/plugin-react';
import { defineConfig } from 'vitest/config';

export default defineConfig(({ mode }) => ({
  plugins: [react()],
  server: {
    fs: { allow: ['..'] },
  },
  ...(mode === 'kit'
    ? {
        build: {
          outDir: 'dist-kit',
          rolldownOptions: { input: 'kit.html' },
        },
      }
    : {}),
  test: {
    environment: 'jsdom',
    globals: true,
    setupFiles: ['./src/test-setup.ts'],
    include: ['src/**/*.test.{ts,tsx}'],
  },
}));
