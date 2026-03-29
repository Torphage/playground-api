# Code Structure

## Overview

The source tree is organized around architectural boundaries.

Top-level modules:

- `api/`
- `application/`
- `domain/`
- `infrastructure/`

Additional files such as `config.rs`, `startup.rs`, and `telemetry.rs` support application bootstrapping and runtime setup.

## Top-level structure

### `src/api/`
HTTP-facing code.

Examples:

- handlers
- route registration
- API state
- API error mapping

Use this layer for transport concerns, not domain logic.

### `src/application/`
Use cases and orchestration.

Examples:

- commands
- queries
- application errors
- ports such as transactions or token generation

Use this layer when implementing what the system does.

### `src/domain/`
Core business concepts.

Examples:

- entities
- value objects
- domain errors
- domain ports such as repositories or password hashing

Use this layer for rules and invariants that should remain independent of infrastructure.

### `src/infrastructure/`
Technical implementations.

Examples:

- Postgres code
- repository implementations
- JWT generation
- Argon2 password hashing

Use this layer when fulfilling ports defined elsewhere.

## Identity example

The `identity` area is a good example of the intended flow.

A registration request might involve:

- `api/handlers/identity/register_user.rs`
- `application/identity/commands/register_user.rs`
- `domain/identity/entities/user.rs`
- `domain/identity/values/*`
- `infrastructure/repositories/identity/users/postgres.rs`

This reflects a general pattern:

- API receives the request
- Application coordinates the use case
- Domain enforces rules
- Infrastructure performs persistence or technical work

## Placement rules

When adding new code, use these rules of thumb.

### Put code in `api/` when it is about:
- HTTP
- route wiring
- request/response handling
- transport-specific error mapping

### Put code in `application/` when it is about:
- use-case orchestration
- commands and queries
- transactions
- calling ports to get work done

### Put code in `domain/` when it is about:
- business rules
- invariants
- entities
- value objects
- domain contracts

### Put code in `infrastructure/` when it is about:
- database access
- crypto implementations
- token implementations
- external systems
- technical adapters

## A few practical rules

- Do not place business rules in handlers
- Do not let infrastructure drive domain design
- Prefer explicit ports over hidden coupling
- Keep modules small and purposeful
- Place code where its reason for change belongs

## Notes

This document is meant to help answer one practical question:

**Where should this code go?**
