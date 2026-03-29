# Deployment

## Overview

`playground-api` is intended to be deployed to a self-hosted machine.

Production uses Docker Compose and is designed so that:

- the API runs in a container
- PostgreSQL runs in a container
- Redis runs in a container
- Caddy can sit in front as a reverse proxy
- secrets stay outside the public repository

## Compose files

Production uses:

- `compose.yml`
- `compose.prod.yml`

## Configuration

The repository contains example configuration files only.

Examples:

- `.env.example`
- `deploy/env/production.env.example`
- `deploy/Caddyfile.example`

Real production secrets and machine-specific values should not be committed.

## Production shape

In production:

- the API should be reached through the reverse proxy
- PostgreSQL should not be exposed publicly
- Redis should not be exposed publicly
- internal service communication should happen over the Docker network

## Running the production stack

```bash
just prod-up
```

## Notes

This document is intentionally brief.
