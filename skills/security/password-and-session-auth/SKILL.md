---
id: password-and-session-auth
name: Password and Session Auth
description: Hash passwords with argon2/bcrypt and make sessions or tokens server-side revocable.
category: security
roles: [security, backend]
compatible_stacks: [rust, axum, postgres]
---
Hash passwords with a modern, salted algorithm (argon2 or bcrypt) — never
MD5, SHA1, or plaintext. Never log or print a raw password, session
cookie, or token, even in debug output. If using session cookies, set
`HttpOnly`, `Secure`, and `SameSite`, and generate the session identifier
with a cryptographically secure random source. If using tokens (JWT or
similar), keep a server-side revocation path (a session/token table checked
on each request, or a short expiry with refresh) — a pure stateless token
with no way to invalidate it on logout is not acceptable for this kind of
auth. The same rule applies to any invite/sharing link token: generate it
with a cryptographically secure random source, not a predictable sequence.
