---
id: sveltekit-component-structure
name: SvelteKit Component Structure
description: Colocate SvelteKit components/stores under src/lib and keep route files thin.
category: frontend
roles: [frontend]
compatible_stacks: [svelte, sveltekit]
---
Keep route files (`src/routes/**/+page.svelte`) thin — they compose
components and wire up data loading, not house business logic. Reusable UI
lives under `src/lib/components`, shared state under `src/lib/stores`, and
API/data-access helpers under `src/lib/api`. Prefer Svelte's built-in
reactivity (`$:`, stores) over introducing a separate state-management
library; reach for a store only when state is genuinely shared across
unrelated routes, not for state a single component or its direct children
already own.
