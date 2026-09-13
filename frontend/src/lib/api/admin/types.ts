export type AdminUserRole = 'user' | 'root';

export interface AdminUser {
  id: string;
  email: string;
  name?: string | null;
  role: AdminUserRole;
  created_at: string;
}
