---
id: server-components-by-default
name: Server Components by Default
description: Default to Server Components; add "use client" only where real interactivity or a browser-only API is needed.
category: frontend
roles: [frontend]
compatible_stacks: [next, react]
---
Server Components are the default. Add `"use client"` only where a component
genuinely needs a browser API or local interactive state — before adding it,
check whether the interactive piece can be isolated into a small leaf client
component instead of promoting an entire section to client-side.
