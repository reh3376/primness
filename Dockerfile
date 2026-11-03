FROM rust:1-bookworm AS build
WORKDIR /src
ENV CARGO_PROFILE_RELEASE_LTO=false
ENV CARGO_PROFILE_RELEASE_CODEGEN_UNITS=16
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY bench ./bench
COPY schemas ./schemas
RUN cargo build --release -p primness-bench --bin primness-dashboard

FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates libssl3 \
    && rm -rf /var/lib/apt/lists/*
COPY --from=build /src/target/release/primness-dashboard /usr/local/bin/primness-dashboard
COPY --from=build /src/schemas /schemas
ENV PRIMNESS_DASHBOARD_PORT=9477
# The container must accept the published port; compose maps it to 127.0.0.1 on the host.
ENV PRIMNESS_DASHBOARD_HOST=0.0.0.0
ENV PRIMNESS_METRICS_DIR=/var/primness/metrics
ENV PRIMNESS_HARDWARE_DIR=/var/primness/hardware
ENV PRIMNESS_SCHEMAS_DIR=/schemas
EXPOSE 9477
ENTRYPOINT ["primness-dashboard"]
