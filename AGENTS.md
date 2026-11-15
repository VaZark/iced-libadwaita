# Agent workflow

Read `CONTRIBUTING.md`, `DESIGN.md`, and `AGENTS_TODO_AND_STATUS.md` before changing code. Preserve user/other-agent work; do not commit, stage, reset, or create branches unless asked.

## Bounded sprint-style implementation and testing

- Work in explicit, finite batches. Order ready tasks first, then by prerequisite dependencies to avoid repeated work, then by difficulty. Reserve disjoint files before delegation; delegate only where it reduces overall investigation/token cost.
- At each sprint start, list the concrete reported issues and acceptance checks in `AGENTS_TODO_AND_STATUS.md`.
- Fix every reported issue in that batch, with root-cause regression tests. Ask the user about ambiguous requirements or subjective API/interaction/design decisions instead of inventing policy. Use Mantine, Bootstrap, and Radix as relevant references for component APIs and primitive/hook separation; preserve this project's layering and controlled-state conventions.
- After core fixes, update the gallery and theme toys to exercise the resulting public APIs and behavior. Do not promote experimental toy-only controls as stable gallery APIs.
- Run targeted tests, then integration/workspace validation. Compilation is not visual or interaction acceptance. Record actual commands, failures, environment limitations, and evidence.
- Save newly discovered issues as the next batch in the report, including dependencies and concrete acceptance checks. Close the current batch with an honest status, then begin the next cycle. Do not get stuck in infinite polishing or unbounded refinement of a single component.
- Keep the report current as tasks finish. Never mark a broad audit complete based only on a few representative tests. Carry unresolved findings forward explicitly.

## Config-owned design values and variants

- Design values come from validated config tokens, including spacing, offsets, geometry, font sizes, gradients, borders, and effects. Do not embed presentation constants or preset-specific arithmetic in rendering logic.
- Named density/size variants (including familiar `sm`, `md`, and `lg`) are config-loaded defaults, not hardcoded renderer behavior. Resolve every declared variant through the same generic per-key inheritance path. Missing or undeclared variants inherit neutral/base values; no name-based magic fallback.
- Legacy convenience enum/API names may identify a config variant for compatibility, but must not define its geometry or restrict custom names. Defaults belong in config; schema validates types and individual bounds.
- Non-shape-changing effects inherit the target's actual shape/geometry. Nested presentation props default to no-ops and must not override parent-owned state, values, security, or events.
