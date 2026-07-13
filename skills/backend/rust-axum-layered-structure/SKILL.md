---
id: rust-axum-layered-structure
name: Rust/Axum Layered Structure
description: Keep a light handlers/services/data separation in an Axum backend without over-generalizing.
category: backend
roles: [backend]
compatible_stacks: [rust, axum]
---
Structure an Axum backend with a light separation of concerns: HTTP
handlers (routing, request/response mapping) stay thin and delegate to a
service/logic layer, which in turn talks to a data layer (repository
functions or queries) rather than embedding SQL directly in handlers. Don't
introduce a trait-per-repository abstraction or dependency-injection
framework unless the project already has multiple real implementations to
swap between — a plain module boundary is enough until that's true.
