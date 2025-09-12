FROM messense/rust-musl-cross:x86_64-musl AS chef
ENV SQLX_OFFLINE=true
RUN cargo install cargo-chef
WORKDIR /url-shortener

FROM chef AS planner
# Copy source code from previous stage
COPY . .
# Generate info for caching dependencies
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /url-shortener/recipe.json recipe.json
# Build & cache dependencies
RUN cargo chef cook --release --target x86_64-unknown-linux-musl --recipe-path recipe.json
# Copy source code from previous stage
COPY . .
# Build application
RUN cargo build --release --target x86_64-unknown-linux-musl

# Create a new stage with a minimal image
FROM scratch
COPY --from=builder /url-shortener/target/x86_64-unknown-linux-musl/release/url-shortener /url-shortener
ENTRYPOINT ["/url-shortener"]
EXPOSE 3000