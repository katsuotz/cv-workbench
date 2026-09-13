import { BackendApiError } from '$lib/api/core/apiError';
import type { AuthApi } from '$lib/api/auth/authApi';
import type { AuthUser } from '$lib/api/auth/types';
import type { CvSessionApi } from '$lib/api/cv-session/cvSessionApi';
import type { CvSessionDraft, CvSessionResponse } from '$lib/api/cv-session/types';

export class AccountService {
  constructor(
    private readonly auth: AuthApi,
    private readonly cvSession: CvSessionApi,
    private readonly getDraft: () => CvSessionDraft,
    private readonly applySession: (session: CvSessionResponse) => void
  ) {}

  currentUser() {
    return this.auth.getCurrentUser();
  }

  googleStartUrl() {
    return this.auth.getGoogleStartUrl();
  }

  linkedInStartUrl(intent: 'login' | 'import' = 'login') {
    return this.auth.getLinkedInStartUrl(intent);
  }

  async authenticate(mode: 'login' | 'register', email: string, password: string, name?: string) {
    const user =
      mode === 'login'
        ? await this.auth.login(email, password)
        : await this.auth.register(email, password, name);
    const session = await this.hydrateOrCreateSession();
    return { user, session };
  }

  logout() {
    return this.auth.logout();
  }

  private async hydrateOrCreateSession() {
    try {
      const session = await this.cvSession.get();
      this.applySession(session);
      return session;
    } catch (error) {
      if (!(error instanceof BackendApiError) || error.status !== 404) throw error;
      const session = await this.cvSession.create(this.getDraft());
      this.applySession(session);
      return session;
    }
  }
}

export type { AuthUser };
