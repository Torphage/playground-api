# Architecture

## Overview

`playground-api` follows a layered architecture with clear boundaries between:

- API
- Application
- Domain
- Infrastructure

The goal is to keep business rules isolated from delivery and technical concerns.

## Layers

### API
Handles HTTP-facing concerns.

Examples:

- routing
- handlers
- request/response mapping
- API-level errors
- application state wiring

The API layer should not contain business rules.

### Application
Coordinates use cases.

Examples:

- commands
- queries
- orchestration
- transactions
- application-level ports

The application layer decides what should happen, but not how low-level details are implemented.

### Domain
Contains the core business model.

Examples:

- entities
- value objects
- domain errors
- domain ports

The domain should contain the rules that matter regardless of framework, transport, or storage.

### Infrastructure
Implements technical details.

Examples:

- database access
- repository implementations
- password hashing
- JWT generation

Infrastructure exists to support the inner layers.

## Dependency direction

Dependencies should point inward:

- API → Application
- Application → Domain
- Infrastructure → Application / Domain contracts

The domain should remain the most stable part of the system.

## Request flow

A typical request flow looks like this:

1. A route is registered in the API layer
2. A handler receives and validates input
3. The handler calls an application command or query
4. The application layer coordinates the use case
5. Domain types enforce business rules
6. Infrastructure implementations handle persistence or external concerns
7. The result is mapped back to an HTTP response

## Design principles

This project aims to follow a few guiding ideas:

- clear boundaries
- explicit dependencies
- small focused modules
- business logic kept away from framework details
- code that is easier to extend and test

## Notes

This document is intentionally brief.

For where code should be placed, see [Code Structure](code-structure.md).
