# Development

## Overview

Local development is intentionally simple:

- run PostgreSQL and Redis with Docker Compose
- run the Rust API on the host

## Common commands

Stop local dependencies:

```bash
just deps-down
```

Run the API locally:

```bash
just dev
```

Run migrations:

```bash
just migrate
```

Run tests:

```bash
just test
```

Run checks:

```bash
just verify
```

## Local setup

Development uses:

- compose.yml
- compose.dev.yml

The API itself is not developed through the Docker service.

That is intentional, to keep iteration fast and debugging straightforward.

## Notes

The Justfile is the preferred entrypoint for day-to-day development tasks.
