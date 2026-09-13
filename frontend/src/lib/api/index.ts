import { AuthApiClient, type AuthApi } from './auth/authApi';
import { CompilationApiClient, type CompilationApi } from './compilation/compilationApi';
import { CvRenderApiClient, type CvRenderApi } from './cv-render/cvRenderApi';
import { CvSessionApiClient, type CvSessionApi } from './cv-session/cvSessionApi';
import { CvImportApiClient, type CvImportApi } from './cv-import/cvImportApi';
import { HttpClient } from './core/httpClient';
import { DocumentsApiClient, type DocumentApi } from './documents/documentsApi';
import { TemplatesApiClient, type TemplatesApi } from './templates/templatesApi';

export interface BackendApi {
  auth: AuthApi;
  cvSession: CvSessionApi;
  cvImport: CvImportApi;
  cvRender: CvRenderApi;
  templates: TemplatesApi;
  documents: DocumentApi;
  compilation: CompilationApi;
}

export function createBackendApi(): BackendApi {
  const http = new HttpClient();
  const documents = new DocumentsApiClient(http);
  const cvSession = new CvSessionApiClient(http, (session) =>
    documents.rememberWorkspace(session.projectId, session.documentId)
  );
  return {
    auth: new AuthApiClient(http),
    cvSession,
    cvImport: new CvImportApiClient(http),
    cvRender: new CvRenderApiClient(http),
    templates: new TemplatesApiClient(http),
    documents,
    compilation: new CompilationApiClient(http)
  };
}
export { BackendApiError } from './core/apiError';
export { SessionContext } from './core/sessionContext';
export { HttpClient } from './core/httpClient';
export type { AuthUser } from './auth/types';
export type { LinkedInPendingImport } from './cv-import/types';
export type { CvSessionDraft, CvSessionResponse } from './cv-session/types';
export type { CvRenderRequest, CvRenderResponse } from './cv-render/types';
export type { CvTemplateSummary } from './templates/types';
export type { DocumentResponse, ProjectResponse } from './documents/types';
export type { ArtifactResponse, CompileJobResponse, DiagnosticResponse } from './compilation/types';
