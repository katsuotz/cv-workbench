# Frontend plan: split LaTeX editor and preview

## Status and repository layout

The first frontend foundation is implemented in `frontend/` as a SvelteKit application. Workspace metadata and the single Bun lockfile remain at the repository root, frontend application and tooling files live under `frontend/`, and backend code belongs under `backend/`.

The app is a SvelteKit workspace with a backend-backed document and compile flow. The application has one preview path: generated TeX is sent to the backend and the returned PDF artifact is displayed with PDF.js. Run frontend scripts from the repository root with `bun run --cwd frontend <script>`.

## Implemented foundation

- SvelteKit route shell and responsive editor/preview workspace.
- Tailwind CSS 4 utility-first styling with global CSS limited to base tokens, resets, and animation primitives.
- CodeMirror-based LaTeX source editor with line highlighting for selected diagnostics.
- Desktop split panes with pointer resizing, keyboard resizing, and reset behavior.
- Mobile editor/preview pane navigation.
- Toolbar with dirty-state indication, preview action, status feedback, and keyboard shortcut.
- Preview states for idle, loading, success, empty, and structured failure.
- Typed `PreviewAdapter` boundary and `BackendPreviewAdapter` for document persistence, compile-job polling, diagnostics, and PDF artifact loading.
- Cookie-backed anonymous and account sessions, server CV draft bootstrap/autosave with optimistic versions, project/document restoration, revision persistence, compile-job polling/cancellation, structured diagnostics, and authenticated PDF artifact loading through the backend API.
- The workspace account panel supports password authentication and Google sign-in through the backend OAuth redirect; pending anonymous edits are flushed before navigation and the callback result is surfaced locally.
- The workspace account panel also supports LinkedIn sign-in. The Summary section can start an OAuth-backed LinkedIn import, review the server-provided pending profile, and replace the current CV only after confirmation.
- PDF.js rendering for backend-produced PDF previews; deterministic Playwright tests use an HTTP fixture that implements the same backend contract and returns a valid PDF.
- Preview controller cancellation and monotonically increasing request IDs so stale results cannot replace newer state; the last successful preview remains available when a later request fails.
- Structured diagnostics and live status announcements.
- Unit/component tests and a Playwright accessibility/workspace smoke-test foundation.
- Domain-specific API modules backed by one shared HTTP client and session context. Authentication, CV sessions, documents, and compilation no longer share a monolithic client implementation.
- Workspace persistence and account orchestration live under `frontend/src/lib/workspace/`; the Svelte component composes the session controller, account service, download helpers, and preview adapter around UI state.
- Reusable controls live under `frontend/src/lib/components/base/`. `Button`, `ButtonLink`, field controls, `RadioCard`, and shared field error/accessibility handling own native semantics and stable IDs while retaining the established `.button`, `.cv-input`, `.field-error`, and template-card visual vocabulary.
- Pure/browser utilities live under `frontend/src/lib/utils/`: filename and string normalization, generic blob downloads, and clipboard helpers are independent of workspace state and can be composed by feature services. Existing workspace download exports remain compatible during the migration.
- Keep base components and utilities free of transport, compiler, CV-session, and workspace orchestration dependencies. Use the repository's Svelte 4/5-compatible legacy syntax until the frontend adopts a coordinated runes migration.
- Keep route entrypoints and the workspace shell focused on orchestration. Feature-specific markup belongs in `components/workspace/` and `landing/`; repeated controls and patterns belong in `components/base/`, with direct native controls reserved for base controls and specialized integrations.
- The backend owns the read-only template catalog and render operation. The frontend loads catalog metadata and first-page preview PDFs through a dedicated templates API, then persists selected and last-generated template IDs with the CV draft.
- LinkedIn import transport remains in a dedicated API module. Imported data is reviewed in a workspace component, applied through the optimistic CV-session boundary, and never placed in callback query parameters.

The adapter seam keeps transport and compiler details out of the Svelte components. Real TeX semantics, package support, PDF loading, and network persistence are provided by the backend integration.

## Goals for the next frontend phases

- Keep source editing and rendered output visibly associated, including useful compiler feedback.
- Preserve the adapter and workspace state boundaries while the backend contract is integrated.
- Keep the responsive and keyboard-accessible workflow usable as the preview representation changes.
- Avoid embedding transport, compiler, or deployment assumptions in Svelte components.

## Architecture and integration boundary

- Keep route-level SvelteKit entry points thin and organize workspace behavior under `frontend/src/lib/`.
- Keep editor, preview, toolbar, diagnostics, and responsive navigation as separate feature components.
- Keep workspace state responsible for source, dirty state, selected pane, split size, request status, last successful result, and diagnostics.
- Keep `frontend/src/lib/api/index.ts` as the API composition root so each workspace uses one transport/session context and domain APIs expose only their own DTOs and operations.
- Keep `frontend/src/lib/workspace/sessionController.ts` responsible for bootstrap, debounced and forced saves, optimistic versions, conflict recovery, and disposal; keep account and browser download concerns in their own services.
- Keep validation and section-specific composition in feature components; reusable field controls should receive a value, error, and optional `data-path`/description and expose the input's native binding and events. Error text must be directly associated through `aria-describedby`/`aria-errormessage` and `aria-invalid`.
- Load the template catalog before CV session bootstrap so the picker has canonical metadata and a valid selection when a saved draft is restored. Keep selected-template changes separate from the last-generated template so stale proof state remains honest.
- Keep preview orchestration dependent on `PreviewAdapter`, not on HTTP or process-spawning details.
- Keep browser tests deterministic with the test-only backend HTTP fixture while application code always uses `BackendPreviewAdapter` and `PUBLIC_API_BASE_URL`.
- Treat backend-generated HTML or artifact content as untrusted at the rendering boundary; define sanitization and artifact-loading rules before integration.

## Backend integration

The backend defines an asynchronous job API, structured diagnostics, bounded PDF artifacts, and the `cv-xelatex` profile. The frontend production adapter persists revisions, polls and cancels jobs, maps diagnostics to editor locations, and loads completed PDF artifacts through PDF.js.

Template integration adds `GET /api/v1/cv/templates`, `GET /api/v1/cv/templates/{id}/preview`, and `POST /api/v1/cv/render`. Rendering failures do not replace the current source or last successful proof. Template preview loading is best-effort and falls back to an accessible metadata card when a PDF cannot be displayed.

## Remaining frontend work

- Add field-level server draft conflict merging and multi-tab coordination if richer collaboration is required; the current local-preserving recovery retries once against the latest optimistic version.
- Add file-aware CodeMirror decorations and previous/next diagnostic movement; direct diagnostic selection already opens the reported source line and column.
- Add document/project navigation only if the product requires multiple documents.
- Expand responsive, keyboard, and screen-reader verification, including browser-level accessibility checks.
- Add CI that runs the frontend checks from the monorepo root.

## Verification expectations

From the repository root, the frontend package should pass:

- `bun run --cwd frontend check`
- `bun run --cwd frontend lint`
- `bun run --cwd frontend test`
- `bun run --cwd frontend test:e2e`
- `bun run --cwd frontend build`

Keep the deterministic backend HTTP fixture for UI tests. Full compile integration tests may use the Compose worker and must not require a developer's host LaTeX installation.

## Acceptance criteria for real integration

- The frontend uses the agreed adapter contract without coupling components to transport details.
- Editing source visibly marks the draft and a user-triggered preview exposes loading, success, empty, and structured error states.
- Stale or failed requests cannot replace a newer successful result, and failed compilation does not erase the last known preview without clear labeling.
- Core workspace actions, pane navigation, resizing, focus, and status announcements remain keyboard accessible.
- Frontend checks and contract-backed integration coverage pass in CI.
