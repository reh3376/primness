FROM rust:1-bookworm AS build
WORKDIR /src
ENV CARGO_PROFILE_RELEASE_LTO=false
ENV CARGO_PROFILE_RELEASE_CODEGEN_UNITS=16
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release --bin primness-dashboard

FROM debian:bookworm-slim
COPY --from=build /src/target/release/primness-dashboard /usr/local/bin/primness-dashboard
ENV PRIMNESS_DASHBOARD_PORT=9477
ENV PRIMNESS_METRICS_DIR=/var/primness/metrics
EXPOSE 9477
ENTRYPOINT ["primness-dashboard"]
