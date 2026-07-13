---
id: docker-compose-for-local-dev
name: Docker Compose for Local Dev
description: Keep docker-compose minimal — real infra dependencies only, config via .env, never commit secrets.
category: backend
roles: [backend, devops]
compatible_stacks: [docker, postgres]
---
Use Docker Compose only for real infrastructure dependencies the app needs
locally (e.g. the database) — don't containerize the app's own build/dev
server unless asked. Configure services (credentials, ports, connection
strings) via environment variables, provide a committed `.env.example`
with placeholder values, and never commit a real `.env` file or hardcoded
credentials into `docker-compose.yml` itself. Give the database service a
healthcheck so dependent services (or a human running `docker compose up`)
can tell when it's actually ready to accept connections, not just started.
