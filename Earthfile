ARG package=three-commas-scraper
FROM busybox

###########################################################################
# BASE IMAGES
###########################################################################

rust:
  FROM rust

  WORKDIR /src
  COPY rust-toolchain.toml .

  RUN dpkg --add-architecture arm64
  RUN apt-get update && apt-get install --no-install-recommends -y gcc-aarch64-linux-gnu libc6-dev-arm64-cross libssl-dev:arm64

  ENV CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc
  ENV AARCH64_UNKNOWN_LINUX_GNU_OPENSSL_LIB_DIR=/usr/lib/aarch64-linux-gnu/
  ENV AARCH64_UNKNOWN_LINUX_GNU_OPENSSL_INCLUDE_DIR=/usr/include/openssl/

  RUN rustup target add x86_64-unknown-linux-gnu
  RUN rustup target add aarch64-unknown-linux-gnu

  SAVE IMAGE --cache-hint

###########################################################################
# CHEF TARGETS
###########################################################################

chef:
  FROM +rust

  RUN cargo install cargo-chef
  SAVE IMAGE --cache-hint

###########################################################################
# PLAN TARGETS
###########################################################################

plan:
  FROM +chef

  COPY . .
  RUN cargo chef prepare --recipe-path recipe.json
  SAVE ARTIFACT recipe.json
  SAVE IMAGE --cache-hint

###########################################################################
# DEPS TARGETS
###########################################################################

deps:
  FROM +chef
  ARG target=x86_64-unknown-linux-gnu

  COPY +plan/recipe.json .
  RUN cargo chef cook --recipe-path recipe.json --target ${target} --release --package ${package}
  SAVE IMAGE --cache-hint

###########################################################################
# BUILD TARGETS
###########################################################################

build:
  FROM +deps
  ARG target=x86_64-unknown-linux-gnu

  COPY --dir . .
  RUN cargo build --target ${target} --release --package ${package} --locked --bin ${package}
  SAVE ARTIFACT target/${target}/release/${package}
  SAVE IMAGE --cache-hint

###########################################################################
# VERSION HELPER
###########################################################################

version:
  FROM +rust

  WORKDIR /src
  COPY --dir . .
  RUN mkdir -p "/out" && cargo pkgid --package ${package} | cut -d# -f2 | cut -d: -f2 > /out/.version

  WORKDIR /out
  RUN echo "version=$(cat .version)"

  SAVE ARTIFACT /out/.version

###########################################################################
# ARTIFACT TARGETS
###########################################################################

amd64-linux-gnu:
  FROM +version
  ENV target=x86_64-unknown-linux-gnu
  ENV platform=amd64-linux-gnu

  COPY --build-arg target=${target} +build/${package} /out/
  RUN mv ${package} "${package}-v$(cat .version)-${platform}"
  RUN sha256sum "${package}-v$(cat .version)-${platform}" > "${package}-v$(cat .version)-${platform}".sha256.txt
  RUN rm .version

  SAVE ARTIFACT /out/*

arm64-linux-gnu:
  FROM +version
  ENV target=aarch64-unknown-linux-gnu
  ENV platform=aarch64-linux-gnu

  COPY --build-arg target=${target} +build/${package} /out/
  RUN mv ${package} "${package}-v$(cat .version)-${platform}"
  RUN sha256sum "${package}-v$(cat .version)-${platform}" > "${package}-v$(cat .version)-${platform}".sha256.txt
  RUN rm .version

  SAVE ARTIFACT /out/*

###########################################################################
# GROUP TARGETS
###########################################################################

amd64:
  COPY +amd64-linux-gnu/* /out/
  # COPY +amd64-linux-gnu-vendored/* /out/
  # COPY +amd64-linux-musl-static/* /out/

  SAVE ARTIFACT /out/*

arm64:
  COPY +arm64-linux-gnu/* /out/
  # COPY +aarch64-linux-gnu-vendored/* /out/
  # COPY +aarch64-linux-musl-static/* /out/

  SAVE ARTIFACT /out/*

all:
  COPY +amd64/* /out/
  COPY +arm64/* /out/

  SAVE ARTIFACT /out/*

###########################################################################
# IMAGE TARGETS
###########################################################################

build-image:
  ARG TARGETPLATFORM
  ARG TARGETVARIANT
  ARG TARGETARCH
  ARG VERSION
  FROM --platform=$TARGETPLATFORM debian:stable-slim

  IF [ "$TARGETARCH" = "amd64" ]
    ENV TARGET=x86_64-unknown-linux-gnu
  ELSE
    ENV TARGET=aarch64-unknown-linux-gnu
  END

  RUN apt-get update \
    && apt-get install -y ca-certificates tzdata \
    && rm -rf /var/lib/apt/lists/*

  EXPOSE 8080
  ENV TZ=Etc/UTC
  ENV APP_USER=appuser

  RUN groupadd $APP_USER \
    && useradd -g $APP_USER $APP_USER

  COPY \
    --platform=linux/amd64 \
    --build-arg target=${TARGET} \
    +build/${package} /usr/bin/${package}-${TARGETARCH}

  RUN chown $APP_USER:$APP_USER /usr/bin/${package}-${TARGETARCH}

  USER $APP_USER
  ENV APP=/usr/bin/${package}-${TARGETARCH}
  CMD ${APP}

  SAVE IMAGE --push ghcr.io/yolodev/${package}:latest
  SAVE IMAGE --push ghcr.io/yolodev/${package}:v${VERSION}

image:
  BUILD --platform=linux/amd64 --platform=linux/arm64 +build-image
