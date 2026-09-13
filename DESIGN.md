# Design system

CV Workbench is a focused CV workbench: the interface should feel like a clear technical desk for turning structured facts into a document.

## Visual direction

- **Mode:** Operate. The UI prioritizes scanability, fast completion, recovery, and confidence in the rendered result.
- **Canvas:** Neutral cool gray (`--canvas`) with white working surfaces and a subtle secondary surface for rails, headers, and tabs. No texture, grain, or decorative background treatment.
- **Ink:** Graphite primary text, muted slate for supporting copy, and quiet slate for metadata. Borders stay 1px and low-contrast except where a stronger divider improves structure.
- **Accent:** One blue accent (`--blue`) for the primary Generate action, active navigation, focus, selected controls, and progress/state indicators. It is not decorative.
- **Semantic states:** Success is green, warning is amber, and compiler/input failure is red. State colors are paired with text and icons, never used alone.
- **Typography:** One sans-serif UI family with a restrained type scale. Monospace is reserved for labels, shortcuts, source, diagnostics, and measured metadata.
- **Shape:** Compact 8px control radius, 12px workspace surfaces, minimal elevation, and consistent button vocabulary. Avoid pill-shaped decoration and heavy shadows.
- **Layout:** Intake is a wide single-task form with section navigation and an obvious Preview action. Workspace is a desktop split editor/preview view; at 900px and below it becomes a single-pane Form/Preview navigation model.
- **Motion:** Short state transitions (150–220ms) for stage reveal, tabs, focus, and feedback. No decorative choreography. All motion yields to `prefers-reduced-motion`.

## Component rules

- Fields use visible labels, a clear required marker, a filled white control surface, and a blue focus ring. Errors sit directly below the field and preserve the user's context.
- Cards group repeatable CV entries but do not become decorative tiles. Entry actions remain compact and secondary.
- Preview is the dominant action. Source, Copy, Download `.tex`, and Download PDF are secondary actions placed near their relevant preview or source state.
- Feedback stays local to the form or preview pane, uses direct language, and avoids persistent success/status labels.
- Compiler diagnostics remain inline and actionable. Selecting a diagnostic can move the user to its source or form context without hiding the message.

## Direction contract

The fourth grounded direction from concept seed `39d4a4b7` is committed: a technical document workbench that keeps the work legible at a glance and makes the current state inspectable. Its first surface begins as intake, then opens the split preview workspace after a valid preview attempt. The signature interaction is the state change from structured input to a rendered document; the honest risk is that a workbench can feel generic, so the blue action language and disciplined section rail must carry the product identity.
