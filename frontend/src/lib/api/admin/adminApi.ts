import { HttpClient } from '../core/httpClient';
import type { AdminUser } from './types';

export interface AdminApi {
  listUsers(): Promise<AdminUser[]>;
}

export class AdminApiClient implements AdminApi {
  constructor(private readonly http: HttpClient = new HttpClient()) {}

  listUsers() {
    return this.http.requestPublic<AdminUser[]>('/api/v1/admin/users');
  }
}
