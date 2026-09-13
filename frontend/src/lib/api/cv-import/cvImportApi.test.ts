import { describe, expect, it, vi } from 'vitest';
import { BackendApiError } from '../core/apiError';
import { CvImportApiClient } from './cvImportApi';

const importedData = {
  identity: { fullName: 'Imported Person', email: 'person@example.com', profiles: [] },
  summary: '',
  experience: [],
  achievements: [],
  skills: [],
  education: [],
  certificates: [],
  projects: []
};

describe('CvImportApiClient', () => {
  it('normalizes pending imports and treats an empty queue as absent', async () => {
    const request = vi
      .fn()
      .mockResolvedValueOnce({ pending_import: { id: 'imp-1', profile: importedData } });
    const api = new CvImportApiClient({ request } as never);

    await expect(api.getPending()).resolves.toEqual({
      id: 'imp-1',
      data: importedData,
      createdAt: null,
      expiresAt: null
    });

    request.mockRejectedValueOnce(new BackendApiError('Not found', 404, 'not_found'));
    await expect(api.getPending()).resolves.toBeNull();
  });

  it('applies with an optimistic version and discards by pending id', async () => {
    const session = { id: 'session-1', version: 4, data: importedData };
    const request = vi.fn().mockResolvedValueOnce(session).mockResolvedValueOnce(undefined);
    const api = new CvImportApiClient({ request } as never);

    await expect(api.apply('imp/1', 3)).resolves.toMatchObject({ id: 'session-1', version: 4 });
    await expect(api.discard('imp/1')).resolves.toBeUndefined();
    expect(request).toHaveBeenNthCalledWith(1, '/api/v1/cv/import/linkedin/pending/imp%2F1/apply', {
      method: 'POST',
      body: JSON.stringify({ expected_version: 3 })
    });
    expect(request).toHaveBeenNthCalledWith(2, '/api/v1/cv/import/linkedin/pending/imp%2F1', {
      method: 'DELETE'
    });
  });
});
