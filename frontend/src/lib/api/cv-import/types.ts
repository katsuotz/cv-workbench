import type { CvData } from '$lib/cv/model';

export interface LinkedInPendingImport {
  id: string;
  data: CvData;
  createdAt?: string | null;
  expiresAt?: string | null;
}
