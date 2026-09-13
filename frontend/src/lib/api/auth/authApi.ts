import { BackendApiError } from '../core/apiError';
import { HttpClient } from '../core/httpClient';
import { SessionContext } from '../core/sessionContext';
import type { AuthResponse, AuthUser } from './types';

export interface AuthApi {
  getCurrentUser(): Promise<AuthUser | null>;
  getGoogleStartUrl(): string;
  getLinkedInStartUrl(intent?: 'login' | 'import'): string;
  login(email: string, password: string): Promise<AuthUser>;
  register(email: string, password: string, name?: string): Promise<AuthUser>;
  logout(): Promise<void>;
}

export class AuthApiClient implements AuthApi {
  constructor(
    private readonly http: HttpClient = new HttpClient(),
    private readonly session: SessionContext = http.sessionContext
  ) {}

  getGoogleStartUrl() {
    return `${this.http.sessionContext.baseUrl}/api/v1/auth/google/start`;
  }

  getLinkedInStartUrl(intent: 'login' | 'import' = 'login') {
    return `${this.http.sessionContext.baseUrl}/api/v1/auth/linkedin/start?intent=${intent}`;
  }

  async getCurrentUser(): Promise<AuthUser | null> {
    try {
      const response = await this.http.requestPublic<AuthResponse>('/api/v1/auth/me');
      this.session.markAuthenticated();
      return response.user ?? (response as unknown as AuthUser);
    } catch (error) {
      if (error instanceof BackendApiError && (error.status === 401 || error.status === 404)) {
        return null;
      }
      throw error;
    }
  }

  async login(email: string, password: string) {
    const response = await this.http.requestPublic<AuthResponse>('/api/v1/auth/login', {
      method: 'POST',
      body: JSON.stringify({ email, password })
    });
    this.session.markAuthenticated();
    return response.user ?? (response as unknown as AuthUser);
  }

  async register(email: string, password: string, name?: string) {
    const response = await this.http.requestPublic<AuthResponse>('/api/v1/auth/register', {
      method: 'POST',
      body: JSON.stringify({ email, password, ...(name?.trim() ? { name: name.trim() } : {}) })
    });
    this.session.markAuthenticated();
    return response.user ?? (response as unknown as AuthUser);
  }

  async logout() {
    await this.http.requestPublic<void>('/api/v1/auth/logout', { method: 'POST' });
    this.session.markLoggedOut();
  }
}
