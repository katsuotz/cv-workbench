# Repository agent instructions

These instructions apply to the whole repository. More specific instructions in a nested `AGENTS.md` take precedence for files below that directory.

## Repository structure

- `frontend/` is the SvelteKit application for CV editing, API-backed persistence, compilation requests, and PDF preview.
- `backend/` is the single Rust/Axum crate containing the HTTP API and compilation worker binaries.
- `migrations/` contains the PostgreSQL migrations used by the backend.
- `docs/` contains maintained frontend and backend architecture plans and backlog notes.

## Product and design references

- Read `PRODUCT.md` before making product, workflow, scope, UX, accessibility, or user-facing behavior decisions.
- Read `DESIGN.md` before making visual, layout, typography, color, component, responsive, or motion decisions.
- Treat those files as the source of truth instead of copying their contents into this file. Preserve their constraints unless the user explicitly requests a change.

## Anti-slop content and UI guardrails

- Every visible string and UI element must serve a concrete user task. If it does not explain, guide, confirm, or help the user act, remove it instead of adding filler.
- Do not generate AI-style garnish: invented eyebrow/kicker labels, motivational status text, fake metrics, decorative badges, redundant helper copy, generic feature claims, or empty states that only restate the page. Avoid phrases such as “ready to start,” “seamless,” “unlock,” “elevate,” “designed for you,” and similar marketing clichés unless the product requirements explicitly call for them.
- Prefer direct, specific product language. Use the user's terminology, name the action and outcome, and avoid inflated claims, vague metaphors, and copy that sounds like a template.
- Do not add visual noise to make a screen feel designed: no gratuitous cards, pills, borders, gradients, oversized hero typography, uppercase microcopy, decorative icons, repeated section labels, or centered filler blocks. Reuse the existing design tokens and component patterns; introduce a new visual treatment only when it solves a documented usability problem.
- Keep hierarchy intentional. Use one clear primary action per context, meaningful status only when state exists, and no duplicate controls or labels. Remove a low-value element rather than rephrasing it.
- Keep text readable across viewports: body and instructional copy should generally be at least 14px, controls and labels at least 11px, and nothing should be reduced below 10px except an explicitly documented decorative detail. Preserve adequate line height, contrast, focus states, and touch targets.
- Before finishing UI work, inspect the result in a real browser at a desktop width and a narrow mobile width. Check for overflow, duplicate or unexplained text, inaccessible controls, and empty visual space. Use the Impeccable skill/audit when available and fix its findings rather than dismissing them without a reason.

## Shared working rules

- Inspect `git status --short` and the relevant diff before editing. Treat existing staged, unstaged, and untracked changes as protected work.
- Work only in files relevant to the current task. Do not reset, restore, clean, overwrite, or remove unrelated changes.
- Keep patches narrow and preserve existing behavior unless the task explicitly requests a behavior change.
- Keep comments to a minimum. Add one only for ambiguous code, non-obvious intent, or a necessary tradeoff.
- Do not add secrets, `.env` files, dependencies, build output, test reports, logs, or generated screenshots to Git.
- Before reporting completion, run `git diff --check` and inspect `git status --short`.
- If a service is started manually on the host, stop it after the task and verify its port is free. Do not stop pre-existing or Docker-managed services without explicit permission.

## Frontend: `frontend/`

### Architecture

- Keep route entry points thin. Put workspace behavior under `frontend/src/lib/`.
- Keep generic leaf UI components under `frontend/src/lib/components/`.
- Keep workspace orchestration under `frontend/src/lib/workspace/`.
- Keep API code under `frontend/src/lib/api/`, composed through `createBackendApi()` in `frontend/src/lib/api/index.ts`.
- Use the domain API modules for authentication, CV sessions, documents, and compilation. Do not reintroduce a monolithic backend client.
- Keep one backend API instance per workspace so anonymous-session caching, authentication state, document context, and transport behavior remain shared.
- Keep `BackendPreviewAdapter` dependent only on document and compilation interfaces. Keep HTTP, authentication, and CV-session details out of preview and UI components.
- Preserve the backend wire contract: cookie credentials, bearer anonymous sessions, one-time `401` retry, snake/camel CV-session normalization, optimistic versions, `204` responses, and binary PDF artifacts.
- Keep workspace session persistence, conflict recovery, autosave scheduling, account orchestration, preview setup, and downloads in their dedicated services/helpers. Keep validation, section navigation, and reactive state in the workspace shell while feature components own markup.
- Keep reusable, presentation-level controls under `frontend/src/lib/components/base/`: `Button`/`ButtonLink`, form fields, `RadioCard`, and shared field error/accessibility helpers. Base controls own their labels, IDs, validation attributes, and native event/value forwarding while preserving the existing `.button`, `.cv-input`, `.field-error`, and related CSS vocabulary.
- Keep pure and browser-bound leaf helpers under `frontend/src/lib/utils/`. Filename/string normalization, blob downloads, and clipboard access belong there; feature services may compose them, but base components must not depend on workspace, API, compiler, or document state.
- Prefer the legacy-compatible Svelte component syntax already used by the frontend (`export let`, `bind:value`, and `on:` forwarding) until a migration explicitly adopts runes. Keep reusable controls independently importable through their base component paths or barrel export.
- Put feature components under feature-specific folders such as `components/workspace/` and `landing/`; keep route entrypoints and `Workspace.svelte` as orchestration shells. Workspace components own markup while the workspace shell retains state, validation, and service coordination.
- Use `components/base` for reusable controls and repeated UI patterns. Direct native controls are limited to base controls and specialized integrations such as CodeMirror and PDF canvas rendering.
- Keep shared pure/browser helpers in `frontend/src/lib/utils/`; leave CV fingerprinting/model logic under `cv/`, API serialization under `api/`, preview logic under `preview/`, and session/account/download orchestration under `workspace/`.

### Frontend commands

Run from the repository root:

```powershell
bun run --cwd frontend check
bun run --cwd frontend lint
bun run --cwd frontend test
bun run --cwd frontend test:e2e
bun run --cwd frontend build
```

For focused development, run the equivalent command from `frontend/` with `bun run <script>`.

### Frontend testing expectations

- Add focused unit tests beside API, workspace, adapter, and controller behavior.
- Preserve deterministic Playwright backend fixtures; browser tests must not require a host XeLaTeX installation.
- When changing API paths or response shapes, update API tests, adapter tests, and affected browser fixtures.
- Preserve keyboard accessibility, responsive pane behavior, status announcements, diagnostics, and the last successful preview state.

## Backend: `backend/`

### Architecture

- Keep the backend as one Rust crate unless a boundary becomes independently reusable or deployable.
- Organize application code by feature. Current feature areas are `sessions`, `documents`, `cv`, and `compilation`.
- Keep feature models, routes, repositories, and services together. Routes handle HTTP translation, services own validation/authorization/use-case coordination, and repositories own PostgreSQL persistence.
- Use repository traits and the `Compiler` trait as explicit dependency-injection boundaries. Avoid abstraction layers without a concrete reuse or testing benefit.
- Keep `latex-renderer-backend` as the API binary, `latex-renderer-worker` as the queue worker, and `seed` as the explicit development seed binary.
- Preserve API semantics, authorization, optimistic CV-session versions, structured diagnostics, bounded artifact handling, and the `cv-xelatex` profile.
- Treat compiler execution as a security boundary: preserve `-no-shell-escape`, time/resource bounds, temporary workspace cleanup, bounded logs/artifacts, and disabled host compilation by default.
- Keep migrations backward-aware and verify database-version assumptions before changing schema or Compose configuration.
- When changing an endpoint or DTO, update the frontend API module and contract coverage in the same task when both are in scope.

### Backend commands

Run from the repository root:

```powershell
cargo fmt --manifest-path backend/Cargo.toml -- --check
cargo check --manifest-path backend/Cargo.toml --locked --all-targets
cargo test --manifest-path backend/Cargo.toml --locked
```

For local backend execution:

```powershell
Set-Location backend
cargo run --bin latex-renderer-backend
cargo run --bin latex-renderer-worker
```

For the database/API/worker stack, use the backend Compose project and its `.env` file:

```powershell
Set-Location backend
docker compose --env-file .env up -d --build
```

Host-side compilation requires explicit `LATEX_COMPILER_ENABLED=true` and a valid `LATEX_COMPILER_PATH`. The Compose worker image provides XeLaTeX and the CV fonts/packages; normal API and database tests should not require a host LaTeX installation.

### Backend testing expectations

- Add unit tests for validation, authorization, serialization, and service behavior.
- Add route/repository/integration coverage when changing persistence or HTTP contracts.
- Verify migrations and optimistic concurrency behavior for schema or CV-session changes.
- Exercise worker/compiler changes with bounded, deterministic fixtures where possible; do not make ordinary tests depend on a developer machine's TeX installation.

## Cross-layer changes

For changes spanning frontend and backend:

1. Identify the affected HTTP route, DTO, authentication/session behavior, and persistence boundary.
2. Update the backend implementation and tests first when the wire contract changes.
3. Update the owning frontend API module and adapter/workspace tests.
4. Run the relevant frontend and backend checks, then the broader suites when risk warrants it.
5. Document durable architecture or contract changes in `docs/` and keep `README.md` commands accurate.
