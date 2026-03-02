# =========================================================================
# STAGE 1: The Builder
# =========================================================================
# We use the official Rust image based on Debian Bookworm.
# This environment contains the compiler, cargo, and all build tools.
FROM rust:1.93-bookworm AS builder

# Set the working directory inside the container
WORKDIR /usr/src/app

# --- Dependency Caching Step ---
# To avoid recompiling all dependencies every time you change your code,
# we copy ONLY the dependency manifests first.
COPY Cargo.toml Cargo.lock ./

# We create a dummy `main.rs` file so Cargo can compile the dependencies.
# This creates a cached Docker layer containing all compiled external crates.
RUN mkdir src && \
    echo "fn main() { println!(\"Dummy!\"); }" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# --- Application Build Step ---
# Now we copy your actual source code. If only your source code changed,
# Docker will skip the dependency build above and start right here.
COPY . .

# Tell sqlx to use the saved `sqlx-data.json` instead of a live database
ENV SQLX_OFFLINE=true

# Touch the main file to force Cargo to recompile the application,
# then build the final release binary.
RUN touch src/main.rs && cargo build --release

# =========================================================================
# STAGE 2: The Runtime
# =========================================================================
# We abandon the 1GB+ Rust image and start fresh with a tiny Debian image.
FROM debian:bookworm-slim AS runtime

# Set the working directory for the runtime container
WORKDIR /app

# Install root certificates (necessary if your app ever makes outgoing HTTPS requests)
# and clean up the package manager cache to keep the image incredibly small.
RUN apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Copy ONLY the compiled binary from the builder stage into this tiny image.
# NOTE: Replace `your_project_name` with the actual name from your Cargo.toml!
COPY --from=builder /usr/src/app/target/release/playground-api /usr/local/bin/app_binary

# Expose the port the app will listen on
EXPOSE 3000

# Set the binary as the default command to run when the container starts
ENTRYPOINT ["app_binary"]
