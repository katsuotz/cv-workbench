import { describe, expect, it } from 'vitest';
import { AuthApiClient } from './authApi';

describe('AuthApiClient', () => {
  it('builds LinkedIn start URLs for login and import intents', () => {
    const auth = new AuthApiClient({ sessionContext: { baseUrl: 'https://cv.example' } } as never);

    expect(auth.getLinkedInStartUrl()).toBe(
      'https://cv.example/api/v1/auth/linkedin/start?intent=login'
    );
    expect(auth.getLinkedInStartUrl('import')).toBe(
      'https://cv.example/api/v1/auth/linkedin/start?intent=import'
    );
  });
});
