set dotenv-load := true
set export := true

default:
    @just --list

# =========================
# VARIABLES
# =========================

compose_base := "docker compose -f compose.yml"
compose_dev  := "docker compose -f compose.yml -f compose.dev.yml"
compose_prod := "docker compose -f compose.yml -f compose.prod.yml"

# =========================
# HELPERS
# =========================

# Print all available recipes
list:
    @just --list

# =========================
# DEVELOPMENT DEPENDENCIES
# =========================

# Start local development dependencies (Postgres + Redis)
deps-up:
    {{compose_dev}} up -d db cache

# Stop local development dependencies
deps-down:
    {{compose_dev}} down

# Restart local development dependencies
deps-restart:
    {{compose_dev}} down
    {{compose_dev}} up -d db cache

# Show logs for local development dependencies
deps-logs:
    {{compose_dev}} logs -f db cache

# Show status for local development dependencies
deps-ps:
    {{compose_dev}} ps

# =========================
# APPLICATION DEVELOPMENT
# =========================

# Run the API locally on the host
dev: deps-up
    cargo run

# Run the API locally with watch mode
watch: deps-up
    cargo watch -x run

# =========================
# DATABASE / MIGRATIONS
# =========================

# Run migrations against the local development database
migrate: deps-up
    sqlx migrate run

# Revert the most recent migration
migrate-revert: deps-up
    sqlx migrate revert

# Add a new migration
migrate-add name:
    sqlx migrate add {{name}}

# Check migration status
migrate-info: deps-up
    sqlx migrate info

# Prepare SQLx offline data
sqlx-prepare: deps-up
    cargo sqlx prepare

# =========================
# QUALITY / TOOLING
# =========================

check:
    cargo check

build:
    cargo build

test:
    cargo test

fmt:
    cargo fmt

fmt-check:
    cargo fmt -- --check

clippy:
    cargo clippy --all-targets --all-features -- -D warnings

# Fast local verification
verify: fmt-check clippy test

# Stricter pre-push / CI-like verification
ci: check verify

# =========================
# PRODUCTION STACK
# =========================

# Build production images
prod-build:
    {{compose_prod}} build

# Start the production stack
prod-up:
    {{compose_prod}} up -d --build

# Stop the production stack
prod-down:
    {{compose_prod}} down

# Restart the production stack
prod-restart:
    {{compose_prod}} down
    {{compose_prod}} up -d --build

# Show production logs
prod-logs:
    {{compose_prod}} logs -f

# Show production service status
prod-ps:
    {{compose_prod}} ps

# Pull newer images where applicable, then recreate
prod-pull:
    {{compose_prod}} pull

# =========================
# SHELL ACCESS
# =========================

# Open a shell in the production API container
prod-shell:
    {{compose_prod}} exec api sh

# Open a psql shell in the production DB container
prod-psql:
    {{compose_prod}} exec db psql -U "$POSTGRES_USER" -d "$POSTGRES_DB"

# Open a Redis CLI session in the production cache container
prod-redis:
    {{compose_prod}} exec cache redis-cli

# =========================
# CLEANUP
# =========================

# Stop dev dependencies and remove anonymous resources
clean-dev:
    {{compose_dev}} down --remove-orphans

# Stop prod stack and remove anonymous resources
clean-prod:
    {{compose_prod}} down --remove-orphans

# Remove Rust build artifacts
clean-build:
    cargo clean
