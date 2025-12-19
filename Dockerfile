# ---- Builder Stage ----
# Use a slim, Debian-based Rust image. It's more common and robust than musl builders.
FROM rust:1.83-slim AS builder

WORKDIR /app

# Install build dependencies.
# openssl-sys (a common dependency) needs libssl-dev and pkg-config.
# Even with rustls, some other library might need a C compiler.
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Cache dependencies.
# Create a dummy project to build only the dependencies.
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --locked --release

# Copy the actual source code and build the application.
COPY ./src ./src
# Touch the source file to ensure the cache is invalidated for our main crate
RUN touch src/main.rs
RUN cargo build --locked --release

# ---- Final Stage ----
# Use a lightweight Debian image for the final container.
FROM debian:12-slim AS final

# Set metadata
LABEL maintainer="Gemini"
LABEL description="Scrapes a VPN subscription link from xcvpn.us."

# Install runtime dependencies. ca-certificates is required for making HTTPS calls.
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create a non-root user and group for security.
RUN addgroup --system app && adduser --system --ingroup app app

# Copy the compiled binary from the builder stage.
COPY --from=builder /app/target/release/vpn-sub-scraper /usr/local/bin/vpn-sub-scraper

# Set the working directory
WORKDIR /app

# Expose the port the app runs on
EXPOSE 3000

# Set required environment variables. These should be provided when running the container.
ENV XCVPN_EMAIL=""
ENV XCVPN_PASSWORD=""
# Set a sane default for logging
ENV RUST_LOG="info"

# Switch to the non-root user.
USER app:app

# Set the entrypoint for the container.
CMD ["/usr/local/bin/vpn-sub-scraper"]
