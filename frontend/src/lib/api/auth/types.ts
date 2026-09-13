export interface AuthUser {
  id: string;
  email: string;
  name?: string | null;
  role: 'user' | 'root';
}

export interface AuthResponse {
  user: AuthUser;
}
