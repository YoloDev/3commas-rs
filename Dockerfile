FROM --platform=$BUILDPLATFORM rust AS base
WORKDIR /src
COPY rust-toolchain.toml .

FROM --platform=$BUILDPLATFORM base as base-amd64
ENV target=x86_64-unknown-linux-gnu
RUN rustup target add ${target}

FROM --platform=$BUILDPLATFORM base as base-arm64
ENV target=aarch64-unknown-linux-gnu
RUN rustup target add ${target}
RUN dpkg --add-architecture arm64 \
  && apt-get update \
  && apt-get install --no-install-recommends -y gcc-aarch64-linux-gnu libc6-dev-arm64-cross libssl-dev:arm64
ENV CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc
ENV AARCH64_UNKNOWN_LINUX_GNU_OPENSSL_LIB_DIR=/usr/lib/aarch64-linux-gnu/
ENV AARCH64_UNKNOWN_LINUX_GNU_OPENSSL_INCLUDE_DIR=/usr/include/openssl/

FROM --platform=$BUILDPLATFORM base-${TARGETARCH} AS chef
RUN cargo install cargo-chef

FROM --platform=$BUILDPLATFORM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM --platform=$BUILDPLATFORM chef AS builder
COPY --from=planner /src/recipe.json recipe.json

# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --target ${target} --recipe-path recipe.json

# Build application
COPY . .
RUN cargo build --package three-commas-scraper --release --target ${target} --locked --bin three-commas-scraper \
  && cp target/${target}/release/three-commas-scraper target/three-commas-scraper

FROM debian:stable-slim
ARG APP=/usr/src/three-commas-scraper

RUN apt-get update \
  && apt-get install -y ca-certificates tzdata \
  && rm -rf /var/lib/apt/lists/*

EXPOSE 8080
ENV TZ=Etc/UTC \
  APP_USER=appuser

RUN groupadd $APP_USER \
  && useradd -g $APP_USER $APP_USER \
  && mkdir -p ${APP}

COPY --from=builder /src/target/three-commas-scraper ${APP}/three-commas-scraper
RUN chown -R $APP_USER:$APP_USER ${APP}

USER $APP_USER
WORKDIR ${APP}

CMD ["./three-commas-scraper"]
