# CV Workbench

CV Workbench is a Bun/Rust monorepo for creating professional CVs. The SvelteKit frontend collects structured CV data, displays generated LaTeX and compiler diagnostics, previews the resulting PDF, and downloads the PDF or source. The Rust/Axum backend persists anonymous and account-backed sessions, documents, revisions, compile jobs, diagnostics, and PDF artifacts.

## Capabilities

- Structured CV editing for Summary, Experience, Achievements, Skills, Education, Certificates, and Projects.
- Backend-owned template catalog with template previews.
- Generated XeLaTeX source with escaping and validation at the backend boundary.
- Asynchronous compilation with queued, running, succeeded, failed, and cancelled job states.
- Structured compiler diagnostics with optional source locations.
- PDF preview through PDF.js and downloads for PDF and .tex files.
- Anonymous bearer sessions, password accounts, Google OIDC sign-in, and optional LinkedIn OIDC sign-in.
- Anonymous-session transfer when an account is created or a supported social sign-in completes.
- Review-before-apply LinkedIn profile import.
- Responsive Form/Preview navigation and Cmd/Ctrl+Enter generation.
- Optimistic autosave with one retry after a version conflict; the current tab remains editable if a save fails.

## Architecture

~~~text
Browser (SvelteKit) ──HTTP──> Rust/Axum API ──> PostgreSQL
                                  │
                                  └── compile job ──> XeLaTeX worker ──> PDF artifact
~~~

~~~text
.
├── frontend/       SvelteKit 5 app with CodeMirror and PDF.js
├── backend/        One Rust 2024/Axum crate, migrations, API, worker, and seed binary
├── deploy/         Production Compose, frontend image, and Nginx configuration
├── loadtest/       k6 backend workflow capacity test
├── docs/           Architecture plans and backlog
├── package.json    Root Bun workspace metadata
└── bun.lock        Shared dependency lockfile
~~~

The backend is organized by feature under backend/src/: sessions, documents, cv, and compilation. Its binaries are latex-renderer-backend, latex-renderer-worker, and seed. Source-controlled CV templates and sample PDFs live under backend/src/cv/templates/.

The API runs SQLx migrations at startup. The worker claims PostgreSQL-backed jobs and runs the cv-xelatex profile with -no-shell-escape, temporary workspaces, application timeouts, and bounded PDF artifacts. Redis is not required.

## Requirements

- Bun 1.4.2
- Rust and Cargo
- Docker Desktop with Compose

The local and production Compose files use PostgreSQL 19 Beta 3, which is pre-release software. The local Compose worker image includes XeLaTeX, the required LaTeX packages, and CV fonts; a host XeLaTeX installation is not required for the normal Docker workflow.

## Local setup

Run these commands from the repository root:

~~~console
bun install
docker compose --env-file backend/.env -f backend/compose.yaml up -d --build
bun run --cwd frontend dev
~~~

Before starting Compose, create frontend/.env and backend/.env by copying their corresponding .env.example files with your operating system's file manager or file-copy command.

The default local endpoints are:

- Frontend: http://localhost:5173
- API: http://localhost:18732
- PostgreSQL: localhost:5434

Check the API from another terminal:

~~~console
curl --fail http://localhost:18732/health/live
curl --fail http://localhost:18732/health/ready
~~~

To stop the local services while preserving the PostgreSQL volume:

~~~console
docker compose --env-file backend/.env -f backend/compose.yaml down
~~~

The API image does not contain TeX. Direct host execution of the worker keeps compilation disabled unless LATEX_COMPILER_ENABLED=true and LATEX_COMPILER_PATH point to an installed XeLaTeX environment. COMPILE_TIMEOUT_SECONDS accepts values from 1 through 90 seconds.

PostgreSQL uses a dedicated latex-renderer-postgres19beta3 volume because beta catalog versions may not be compatible across releases.

## Configuration

Frontend values are read at build time from frontend/.env:

| Variable | Local example | Purpose |
| --- | --- | --- |
| PUBLIC_API_BASE_URL | http://localhost:18732 | Backend base URL |
| PUBLIC_SITE_URL | http://localhost:5173 | Canonical site and metadata URL |
| PUBLIC_LINKEDIN_ENABLED | true | Shows or hides LinkedIn features |
| PUBLIC_GOOGLE_TAG_ID | empty | Optional production analytics ID |

Backend values are read from backend/.env. The example file includes database connection, CORS origin, API binding, session, compiler, cookie, Google, and LinkedIn settings. Each provider requires its client ID, client secret, and redirect URI; provider setup and callback requirements are documented in [backend/README.md](backend/README.md).

## Commands and tests

Run frontend commands from the repository root:

~~~powershell
bun run --cwd frontend check
bun run --cwd frontend lint
bun run --cwd frontend test
bun run --cwd frontend test:e2e
bun run --cwd frontend build
~~~

Run backend checks from the repository root:

~~~powershell
cargo fmt --manifest-path backend/Cargo.toml -- --check
cargo check --manifest-path backend/Cargo.toml --locked --all-targets
cargo test --manifest-path backend/Cargo.toml --locked
~~~

Playwright starts the frontend and a deterministic HTTP backend fixture on 127.0.0.1:18733. Browser tests therefore do not require PostgreSQL or XeLaTeX. Install the Chromium browser once if needed:

~~~powershell
bunx playwright install chromium
~~~

The explicit development seed creates or updates an account and deterministic demo CV data. Run it with the backend environment loaded:

~~~console
cd backend
cargo run --bin seed -- --email demo@example.test --password 'a-long-development-password'
~~~

See [loadtest/README.md](loadtest/README.md) for the k6 API workflow capacity test.

## Deployment

Production runs the PostgreSQL database, Rust API, XeLaTeX worker, and static frontend as Docker Compose services. The frontend is served by Nginx; host Nginx terminates HTTPS and proxies /api/, /health/, and frontend traffic to the services. The API and frontend are bound to loopback ports 18732 and 18731, and production uses COOKIE_SECURE=true.

Pushing a semantic version tag such as v0.1.7 triggers [.github/workflows/release.yml](.github/workflows/release.yml). The workflow validates the frontend and backend, builds and pushes the frontend/API/worker images to GHCR, deploys over SSH, waits for service health checks, and verifies the public HTTPS endpoints. Configure the deployment secrets listed in [deploy/README.md](deploy/README.md) before using the release workflow.

See [deploy/README.md](deploy/README.md), [deploy/compose.production.yaml](deploy/compose.production.yaml), and [deploy/nginx-cvworkbench.conf](deploy/nginx-cvworkbench.conf) for production configuration.

## Current limits

The following areas remain unfinished: password reset and email verification, active-process cancellation, complete compiler resource and filesystem isolation, compiler log bounding, artifact/session cleanup jobs, quotas and rate limits, and richer multi-document navigation. PostgreSQL 19 Beta 3 should also be revisited before treating it as a long-term production dependency.

See [docs/what-left-to-do.md](docs/what-left-to-do.md) for the maintained backlog, [docs/frontend-plan.md](docs/frontend-plan.md) for frontend architecture, and [docs/backend-plan.md](docs/backend-plan.md) for backend architecture.

Do not commit .env files, dependencies, build output, test reports, logs, or generated screenshots.
