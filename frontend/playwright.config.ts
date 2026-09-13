import { defineConfig, devices } from '@playwright/test';

export default defineConfig({
  testDir: './tests/e2e',
  webServer: [
    {
      command: 'node tests/e2e/mock-backend.mjs',
      port: 18733,
      reuseExistingServer: false
    },
    {
      command: 'bun run dev --host 127.0.0.1',
      port: 5173,
      reuseExistingServer: false,
      env: {
        PUBLIC_API_BASE_URL: 'http://127.0.0.1:18733',
        PUBLIC_LINKEDIN_ENABLED: 'true'
      }
    }
  ],
  use: { baseURL: 'http://127.0.0.1:5173', trace: 'on-first-retry' },
  projects: [{ name: 'chromium', use: { ...devices['Desktop Chrome'] } }]
});
