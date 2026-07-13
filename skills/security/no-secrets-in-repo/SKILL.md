---
id: no-secrets-in-repo
name: No Secrets in Repo
description: Never commit API keys, tokens, or secrets to the repository.
category: security
roles: [security, backend, frontend]
compatible_stacks: []
---
Never commit API keys, tokens, or secrets to the repository. Before adding
any credential-like value, confirm it's read from an environment variable or
a secret store, not hardcoded or checked into a config file tracked by git.
