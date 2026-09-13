# What’s left to do

The core CV workflow is implemented: the SvelteKit frontend provides structured intake, generated LaTeX source, PDF preview, diagnostics, downloads, and responsive editor/preview navigation. The Rust backend provides anonymous and account sessions, persisted CV drafts and document revisions, queued XeLaTeX jobs, cancellation requests, and bounded PDF artifacts. Frontend unit/component tests and deterministic Playwright smoke tests are in place.

This list reflects the remaining work in the current repository.

## Product and workflow

- [ ] Decide whether to add browser-local or offline draft recovery. Current drafts autosave to the backend, and editing continues in the current tab when remote autosave fails.
- [ ] Add field-level optimistic-conflict merging and multi-tab coordination if users need concurrent editing. The current flow preserves the local draft and retries it once against the latest saved version, but it does not merge concurrent field changes.
- [ ] Add richer compiler-diagnostic navigation, including file-aware decorations and direct previous/next movement. Selecting a diagnostic now opens the generated source at its reported line and column.
- [ ] Add document and project navigation only if the product expands beyond the current single-CV workflow.
- [ ] Define whether collaboration, sharing, or a broader document model belongs on the roadmap.

## Authentication and account lifecycle

- [ ] Add password reset and email verification if accounts become a supported production workflow.
- [ ] Decide whether MFA, account deletion, and role-based administration are required.
- [ ] Define session and anonymous-data retention rules, including what happens when an anonymous session expires without registration.
- [ ] Obtain and maintain the approved LinkedIn profile-data access required for full-history import; ordinary self-service LinkedIn OIDC access only supplies lite identity data.

## Compiler reliability and security

- [ ] Make cancellation stop an active compiler process. The API records cancellation requests and queued jobs can be cancelled, but a running XeLaTeX process is currently observed only after it returns or reaches its timeout.
- [ ] Add process/container resource enforcement beyond the application timeout, including CPU, memory, and process limits.
- [ ] Define and enforce compiler filesystem and network policy. The current worker uses temporary directories and `-no-shell-escape`, but isolation policy is not yet a complete production boundary.
- [ ] Bound compiler stdout/stderr capture and add deterministic worker/compiler integration fixtures.
- [ ] Add artifact cleanup and session cleanup jobs. Artifacts receive a seven-day expiry, but expired rows and old sessions are not currently swept.
- [ ] Add per-account or per-session quotas and request/rate limits.

## Testing and operations

- [ ] Add CI for frontend check, lint, unit tests, end-to-end tests, and build.
- [ ] Add CI for Rust formatting, locked checks/tests, migrations, and backend route/repository/worker integration coverage.
- [ ] Expand browser accessibility coverage beyond the current automated Axe smoke test, including keyboard flows and a screen-reader pass.
- [ ] Add metrics and error reporting for request failures, compile latency, queue depth, cancellations, timeouts, artifact usage, and worker resource use.
- [ ] Establish deployment targets, production environment configuration, secrets handling, database migration policy, and backup/restore procedures.
- [ ] Define user-facing support and recovery flows for failed saves, expired sessions, failed compilation, and unavailable workers.

## Deferred infrastructure

- [ ] Introduce Redis only if distributed queueing, rate limiting, or coordination requirements justify it.
- [ ] Revisit the PostgreSQL 19 beta development dependency before production deployment.
