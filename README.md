# LaTeX Renderer

LaTeX Renderer is a pnpm/Rust monorepo for editing LaTeX documents and previewing compiled PDFs. The current product focuses on CV workflows, with a SvelteKit frontend and a Rust/Axum backend that stores documents and runs isolated XeLaTeX jobs.

## Repository layout

```text
.
├── frontend/       SvelteKit, Svelte 5, Tailwind CSS 4, CodeMirror, PDF.js
├── backend/        Rust/Axum API and PostgreSQL-backed compilation worker
├── docs/           Architecture plans and backlog
├── package.json    Root pnpm workspace metadata
└── pnpm-lock.yaml  Shared dependency lockfile
```

The backend is a single Rust crate organized by feature. Redis is not required.

## Requirements

- Node.js with pnpm 10
- Rust and Cargo
- Docker Desktop with Compose

The development stack uses PostgreSQL 19 Beta 3, which is pre-release software.

## Local setup

Install frontend dependencies from the repository root:

```powershell
pnpm install
```

Create local environment files:

```powershell
Copy-Item frontend/.env.example frontend/.env
Copy-Item backend/.env.example backend/.env
```

Start PostgreSQL, the API, and the XeLaTeX worker:

```powershell
Set-Location backend
docker compose --env-file .env up -d --build
Set-Location ..
```

The default local endpoints are:

- Frontend: `http://localhost:5173`
- API: `http://localhost:18732`
- PostgreSQL: `localhost:5434`

Start the frontend in another terminal:

```powershell
pnpm --dir frontend dev
```

Check that the API is running:

```powershell
Invoke-WebRequest http://localhost:18732/health/live
Invoke-WebRequest http://localhost:18732/health/ready
```

## Commands

Run these from the repository root.

Frontend:

```powershell
pnpm --dir frontend check
pnpm --dir frontend lint
pnpm --dir frontend test
pnpm --dir frontend test:e2e
pnpm --dir frontend build
```

Backend:

```powershell
cargo fmt --manifest-path backend/Cargo.toml -- --check
cargo check --manifest-path backend/Cargo.toml --locked --all-targets
cargo test --manifest-path backend/Cargo.toml --locked
```

The frontend renders returned PDF artifacts with PDF.js. End-to-end tests use a deterministic backend fixture, so they do not require a local XeLaTeX installation.

The backend worker runs XeLaTeX with bounded time and output limits, `-no-shell-escape`, temporary workspaces, and cleanup.

## Project status

The core editor, PDF preview, anonymous sessions, document persistence, compile jobs, diagnostics, and CV-oriented XeLaTeX profile are implemented. Production deployment, account recovery, stronger isolation, retention policies, and richer project navigation remain on the backlog.

See [docs/what-left-to-do.md](docs/what-left-to-do.md) for the backlog, [docs/frontend-plan.md](docs/frontend-plan.md) for frontend architecture, and [docs/backend-plan.md](docs/backend-plan.md) for backend architecture.

## Development notes

Do not commit `.env` files, dependencies, build output, test reports, logs, or generated screenshots. PostgreSQL Beta 3 uses a dedicated Compose volume because beta catalog versions may not be compatible across releases.
