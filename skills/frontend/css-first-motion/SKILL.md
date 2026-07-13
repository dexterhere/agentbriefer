---
id: css-first-motion
name: CSS-first Motion
description: Prefer CSS-first animation over JS-driven animation, and always respect prefers-reduced-motion.
category: frontend
roles: [frontend]
compatible_stacks: [next, react, css]
---
Prefer CSS-first animation (keyframes/transitions defined in CSS) over
JS-driven animation, except inside components that are already client
components. Every animation must respect `prefers-reduced-motion` — provide
a reduced-motion variant or a no-op fallback, never ship motion that ignores
it.
