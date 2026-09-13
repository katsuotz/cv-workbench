# Backend implementation plan

## Status

The first backend slice is implemented under `backend/`. It uses Rust, Axum, Tokio, SQLx, and PostgreSQL 19-compatible migrations. Redis is intentionally not used.

The service currently provides anonymous sessions, persisted projects/documents/revisions, PostgreSQL-backed compile jobs, structured diagnostics, artifact storage, health endpoints, and a separate worker binary. The Compose worker image includes XeLaTeX and the CV package/font set; host-side compilation remains disabled by default.

## Architecture

- `latex-renderer-backend` serves the HTTP API and runs database migrations.
- `latex-renderer-worker` claims jobs with PostgreSQL row locking and executes the compiler boundary.
- Source code is organized by feature: `sessions`, `documents`, `compilation`, and backend-owned CV templates.
- PostgreSQL stores active template catalog metadata and ordering, while source-controlled `.tex` files and fixed sample PDFs live under `cv/templates/`.
- Each feature keeps its models, routes, repository, and service together. Repositories own SQLx persistence, while services own validation, authorization, and use-case coordination.
- Repository traits and the existing `Compiler` trait are the explicit dependency-injection boundaries. Concrete PostgreSQL repositories and services are composed in `lib.rs`; no workspace crates are needed yet.
- PostgreSQL generates UUIDv7 identifiers with `uuidv7()` and stores documents, jobs, diagnostics, and bounded PDF artifacts.
- Account sessions support password authentication and Google OpenID Connect authorization-code sign-in. Google identities are keyed by the provider subject, and verified email matching links them to an existing account.
- The first compile profile is `cv-xelatex`; the worker invokes XeLaTeX with `-no-shell-escape`, bounded time, temporary workspaces, and cleanup.
- Redis is deferred until distributed queue or rate-limit requirements justify it.

### Backend source layout

```text
backend/src/
├── main.rs
├── lib.rs
├── config.rs
├── error.rs
├── sessions/
│   ├── mod.rs
│   ├── model.rs
│   ├── google.rs
│   ├── repository.rs
│   ├── service.rs
│   └── routes.rs
├── documents/
│   ├── mod.rs
│   ├── model.rs
│   ├── repository.rs
│   ├── service.rs
│   └── routes.rs
├── compilation/
│   ├── mod.rs
│   ├── model.rs
│   ├── repository.rs
│   ├── service.rs
│   ├── compiler.rs
│   ├── worker.rs
│   └── routes.rs
└── cv/
    ├── templates/
    │   ├── editorial-v1.tex
    │   ├── compact-v1.tex
    │   ├── modern-v1.tex
    │   ├── editorial-v1.pdf
    │   ├── compact-v1.pdf
    │   └── modern-v1.pdf
    ├── model.rs
    ├── repository.rs
    ├── routes.rs
    └── service.rs
```

The backend remains one crate. Additional workspace crates should be introduced only when a boundary becomes independently reusable or independently deployable.

## API boundary

- `GET /health/live`
- `GET /health/ready`
- `POST /api/v1/sessions/anonymous`
- `GET /api/v1/auth/google/start`
- `GET /api/v1/auth/google/callback`
- `POST /api/v1/auth/register`
- `POST /api/v1/auth/login`
- `POST /api/v1/auth/logout`
- `GET /api/v1/auth/me`
- `POST /api/v1/projects`
- `POST /api/v1/projects/{project_id}/documents`
- `GET /api/v1/documents/{document_id}`
- `PUT /api/v1/documents/{document_id}`
- `POST /api/v1/documents/{document_id}/compile`
- `GET /api/v1/compile-jobs/{job_id}`
- `DELETE /api/v1/compile-jobs/{job_id}`
- `GET /api/v1/artifacts/{artifact_id}`
- `GET /api/v1/cv/templates`
- `GET /api/v1/cv/templates/{id}/preview`
- `POST /api/v1/cv/render`

Compile jobs return queued/running/succeeded/failed/cancelled states and structured diagnostics with optional file, line, and column locations. Successful jobs expose PDF artifact metadata and an authenticated artifact endpoint.

`POST /api/v1/cv/render` accepts typed CV data and an active `template_id`, applies backend LaTeX escaping, enforces the 1 MiB data and 512 KiB source limits, and returns generated source and timestamp. `cv_drafts.generated_template_id` records which template produced persisted source; `template_id` remains the current selection. The legacy `default` identifier is normalized to `editorial-v1`.

Google sign-in uses the same account cookie and requests only `openid email profile`. The backend validates state, nonce, S256 PKCE, the Google issuer and audience, token lifetime, immutable provider subject, and `email_verified`. A successful Google sign-in creates or links an account and transfers the current anonymous project atomically. Google-only accounts have no usable password until a future password-setting flow is added.

## Remaining backend work

- Add process/container resource enforcement beyond the application timeout.
- Add real cancellation of an active compiler process and bounded log capture.
- Add artifact retention cleanup and anonymous quota enforcement.
- Add deployment configuration, metrics, CI, and PostgreSQL integration tests.
- Continue frontend integration hardening, richer diagnostic navigation, and production deployment checks.
- Add password reset, password setup for Google-only accounts, email verification, MFA, and role-based administration only if the product requires them.

## Verification

Run from the repository root:

```powershell
cargo fmt --manifest-path backend/Cargo.toml -- --check
cargo check --manifest-path backend/Cargo.toml
cargo test --manifest-path backend/Cargo.toml
docker compose --env-file .env up -d
```

The `backend/compose.yaml` file uses the official PostgreSQL 19 Beta 3 image, the slim Rust API target, and a separate XeLaTeX-enabled worker target under the `latex-renderer` project name. Only the worker target contains TeX packages and enables compilation. PostgreSQL 19 remains pre-release, so this is for development and compatibility testing only. Beta 3 uses its own Docker volume because beta catalog versions are not directly compatible. The migration rejects servers below major version 19.
