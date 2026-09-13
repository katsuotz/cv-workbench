# LaTeX Renderer backend

Rust/Axum service for persisted CV documents and isolated LaTeX compilation.

The source is feature-based: `sessions`, `documents`, `cv`, and `compilation`. The CV feature owns the PostgreSQL-backed template catalog, source-controlled templates under `src/cv/templates/`, and fixed sample PDF previews. Each feature contains its models, routes, repository, and service. PostgreSQL repositories implement small feature-specific traits, and services are constructed with those traits so the HTTP and worker entry points share the same use cases without adding unnecessary abstraction layers.

## Local development

Start the backend stack from the repository root:

```powershell
docker compose --env-file .env up -d
```

The API host port is configured in `.env` via `API_PORT` and defaults to `18732`. `FRONTEND_ORIGIN` controls the single browser origin allowed by CORS. Keep `../frontend/.env` in sync if you change the API port. Run Compose from this directory so it reads the backend `.env` directly.

`COOKIE_SECURE=true` adds the `Secure` attribute to the `lr_session` cookie; enable it when the API is served over HTTPS. State-changing requests carrying cookies are accepted only when their `Origin` matches `FRONTEND_ORIGIN`. Browser clients must send credentials for account sessions.

Beta 3 uses a separate Docker volume because PostgreSQL beta catalog versions are not directly compatible across beta releases. Existing data must be migrated with PostgreSQL upgrade tooling or `pg_dump`/`pg_restore`.

Run the API:

```powershell
Set-Location backend
cargo run --bin latex-renderer-backend
```

Run the worker:

```powershell
Set-Location backend
cargo run --bin latex-renderer-worker
```

Host-side worker runs keep compilation disabled unless `LATEX_COMPILER_ENABLED=true` and `LATEX_COMPILER_PATH` point to an installed XeLaTeX environment. The Compose worker uses the dedicated `worker` image target, which contains XeLaTeX, LaTeX extras, and CV-oriented fonts and enables compilation. The API image does not contain TeX. Normal API and database tests do not require TeX.

`COMPILE_TIMEOUT_SECONDS` must be between 1 and 90 seconds. This keeps the compiler timeout safely below the worker's two-minute stale-job recovery threshold.

## API

- `GET /health/live`
- `GET /health/ready`
- `POST /api/v1/sessions/anonymous`
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
- `GET /api/v1/cv/session`
- `POST /api/v1/cv/session`
- `PUT /api/v1/cv/session`
- `GET /api/v1/cv/templates`
- `GET /api/v1/cv/templates/{id}/preview`
- `POST /api/v1/cv/render`

Account registration and login use Argon2id password hashes and set an HttpOnly `lr_session` cookie. Registering while an anonymous session is supplied transfers that session's projects, documents, revisions, and CV draft to the new account atomically. Anonymous bearer sessions remain supported for existing clients.

CV draft writes use an optimistic `expected_version` field. Send `expected_version: 0` to create the first draft, then send the version returned by the previous write; a stale version returns `409 Conflict`. Draft data is bounded to 1 MiB and responses include the associated project, document, and latest revision metadata. `template_id` stores the selected catalog template; `generated_template_id` records which template produced the persisted source, so changing the selection leaves an older preview explicitly outdated. The render endpoint returns escaped source and its timestamp; it does not enqueue a live compile job.

All application identifiers are PostgreSQL UUIDv7 values generated with `uuidv7()`.

## Explicit seed command

The seed binary idempotently creates or updates the explicitly requested account and populates one deterministic Demo CV project, document, revision, and draft. It is never run by migrations or application startup:

```powershell
cargo run --bin seed -- --email admin@example.test --password 'a-long-development-password'
```

The equivalent environment variables are `SEED_EMAIL` and `SEED_PASSWORD`.
