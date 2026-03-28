# Tell Just to automatically load the .env file
set dotenv-load := true

# Default command if you just type `just`
default:
    @just --list

# ==========================================
# DEVELOPMENT
# ==========================================

# Start the database and cache in the background, then run the Rust app natively
dev: db-up
    sqlx migrate run
    cargo run


# Start ONLY the infrastructure (Database & Redis)
db-up:
    docker compose up -d db cache

# Shut down the local infrastructure
db-down:
    docker compose down

# ==========================================
# SQLX / OFFLINE PREPARATION
# ==========================================

# Update the SQLx offline query cache (Run this before committing or building prod!)
# It requires the database to be running.
db-prepare: db-up
    cargo sqlx prepare

# ==========================================
# PRODUCTION
# ==========================================

# Generate the SQLx cache, then build the production Docker images
build-prod: db-prepare
    docker compose -f docker-compose.yml -f docker-compose.prod.yml build

# Spin up the entire production stack (DB, Cache, and compiled API)
up-prod:
    docker compose -f docker-compose.yml -f docker-compose.prod.yml up -d