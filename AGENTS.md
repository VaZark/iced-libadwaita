# Agent instructions

## Context

- Read `CONTRIBUTING.md` for repository conventions.
- Read `AGENTS_TODO_AND_STATUS.md` for the active batch, validation and handoff; tasks live one per file in `TODO/<epic>/` (Todo app vault format).
- User decisions are inline `DECISION (user, <date>)` comments at the code they govern; find them with `grep -rn 'DECISION (' src theme-toys` before re-asking.
- Read `DESIGN.md` only for GUI or public component API changes.
- Consult `ICED_LIMITATIONS.md` when modifying iced integrations, implementing affected features, or upgrading iced-family dependencies.
- Read relevant sections and source files only; avoid unrelated history or full-file dumps.

## Working rules

- Preserve user and other-agent changes. Do not stage, commit, reset, or create branches unless requested.
- Work in finite batches with explicit scope and acceptance checks.
- Order work by readiness, dependencies, then difficulty.
- Fix root causes and add regression coverage where practical.
- Ask before making unresolved product, API, or interaction decisions. Do not invent policy.
- Avoid unrelated refactors and unbounded polishing. Reforatting changes are not a problem
- Never claim tests, visual verification, or audits were completed without evidence.

## Architecture invariants

- Presentation values and named variants are config-owned, validated, and resolved through generic per-key inheritance.
- Do not hardcode preset behavior, presentation constants, or variant-name fallbacks in rendering logic.
- Undeclared variants inherit neutral/base values.
- Effects preserve target geometry. Nested presentation props must not override parent-owned state, security, values, or events.
- Preserve project layering, controlled-state conventions, and public API compatibility unless explicitly changing them.

## Execution

- Use the relevant skill for specialized workflows.
- Run targeted tests before broader workspace validation.
- Compilation does not establish visual or interaction correctness.
- Record actual validation evidence and handoff in `AGENTS_TODO_AND_STATUS.md`; record unresolved work as `TODO/` task files. Keep no history log.
- Finish with a concise summary of changes, verification, and remaining issues.
