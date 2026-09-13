# Product

<!-- impeccable:product-schema 1 -->

## Platform

web

## Users

People creating or updating a professional CV in a focused work session, often switching between structured editing and checking the final document.

## Product Purpose

CV Workbench helps a user collect the facts of their career, generate a polished CV as XeLaTeX, and inspect the compiled result before downloading it. Success is a usable CV and a trustworthy document produced without hiding validation or compiler problems.

## Positioning

The builder keeps the structured CV data, exact generated LaTeX source, compiler diagnostics, and rendered PDF in one workflow so the user can move from intake to a finished document without leaving the tool.

## Operating Context

The primary workflow is a short, form-led intake followed by a split editor and rendered-document workspace. Users edit section-by-section, preview on demand, inspect source when needed, and download PDF or `.tex` output.

## Capabilities and Constraints

- CV sections are Summary, Experience, Achievements, Skills, Education, Certificates, and Projects.
- Required fields are validated before generation and the first invalid field receives focus.
- Changes autosave locally when storage is available; editing continues when autosave fails.
- A backend preview adapter compiles generated LaTeX and returns PDF data or actionable diagnostics.
- Saved documents with generated source reopen with their preview workspace available.
- Users can register with a password or sign in with Google; verified Google sign-in can transfer the current anonymous CV into the account.
- Preserve existing keyboard generation (`Cmd/Ctrl + Enter`), mobile Form/Preview navigation, downloads, source actions, storage schema, generator, preview adapter, and public component behavior.
- Account authentication changes must preserve anonymous editing, password authentication, cookie-backed sessions, and the existing CV session contract.

## Evidence on Hand

The current implementation and tests in `frontend/src/lib/components` and `frontend/tests` are the source of truth for current fields, workflows, labels, and test-facing actions. No external brand assets are required for this work.

## Product Principles

- Start with the work the user must finish now.
- Make required input and recovery paths obvious.
- Treat the rendered PDF as the document outcome, not decoration.
- Keep exact source and diagnostics available without competing with the main workflow.
- Preserve the user's work through local persistence and resilient editing.

## Accessibility & Inclusion

Preserve accessible labels, semantic regions, keyboard behavior, visible focus states, reduced-motion support, and responsive use at narrow and intermediate widths.
