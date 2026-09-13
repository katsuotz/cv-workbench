import { BackendApiError } from '../core/apiError';
import { HttpClient } from '../core/httpClient';
import { normalizeCvSession } from '../cv-session/serialization';
import type { CvSessionResponse } from '../cv-session/types';
import type { LinkedInPendingImport } from './types';

export interface CvImportApi {
  getPending(): Promise<LinkedInPendingImport | null>;
  apply(id: string, expectedVersion: number): Promise<CvSessionResponse>;
  discard(id: string): Promise<void>;
}

export class CvImportApiClient implements CvImportApi {
  constructor(private readonly http: HttpClient = new HttpClient()) {}

  async getPending() {
    try {
      const response = await this.http.request<unknown>('/api/v1/cv/import/linkedin/pending');
      return normalizePendingImport(response);
    } catch (error) {
      if (error instanceof BackendApiError && error.status === 404) return null;
      throw error;
    }
  }

  async apply(id: string, expectedVersion: number) {
    const response = await this.http.request<unknown>(
      `/api/v1/cv/import/linkedin/pending/${encodeURIComponent(id)}/apply`,
      {
        method: 'POST',
        body: JSON.stringify({ expected_version: expectedVersion })
      }
    );
    return normalizeCvSession(response);
  }

  discard(id: string) {
    return this.http.request<void>(`/api/v1/cv/import/linkedin/pending/${encodeURIComponent(id)}`, {
      method: 'DELETE'
    });
  }
}

function normalizePendingImport(value: unknown): LinkedInPendingImport {
  const wrapper = value && typeof value === 'object' ? (value as Record<string, unknown>) : {};
  const pending =
    wrapper.pending && typeof wrapper.pending === 'object'
      ? (wrapper.pending as Record<string, unknown>)
      : wrapper.pending_import && typeof wrapper.pending_import === 'object'
        ? (wrapper.pending_import as Record<string, unknown>)
        : wrapper;
  const data = pending.data ?? pending.cvData ?? pending.cv_data ?? pending.profile;
  if (!data || typeof data !== 'object') {
    throw new Error('The LinkedIn import response did not include CV data.');
  }
  return {
    id: String(pending.id ?? pending.importId ?? pending.import_id ?? ''),
    data: data as LinkedInPendingImport['data'],
    createdAt: (pending.createdAt ?? pending.created_at ?? null) as string | null,
    expiresAt: (pending.expiresAt ?? pending.expires_at ?? null) as string | null
  };
}
