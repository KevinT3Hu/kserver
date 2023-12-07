FROM lukemathwalker/cargo-chef:latest AS chef
WORKDIR /app

# Prepare the build environment
FROM chef as planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Build the project
FROM chef as builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release

# Runtime
FROM debian:buster-slim AS runtime
WORKDIR /app
COPY --from=builder /app/target/release/kserver /usr/local/bin/kserver
# Run server
CMD ["kserver"]