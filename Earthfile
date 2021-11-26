VERSION 0.6

ARG USERPLATFORM=linux/amd64
ARG package=three-commas-scraper
ARG openssl=openssl-3.0.0

FROM scratch

###########################################################################
# RUST
###########################################################################

rust:
  ARG TARGETARCH
  ARG TARGETVARIANT
  ARG MUSL
  FROM "./build/rust-builder+base-${TARGETARCH}${TARGETVARIANT:+-${TARGETVARIANT}}${MUSL:+-musl}"

  COPY rust-toolchain.toml .
  RUN rustup target add "${target}"

  COPY ./build/rust-builder+chef/cargo-chef /usr/bin/cargo-chef
  SAVE IMAGE --push "ghcr.io/yolodev/${package}-build-cache:rust-${TARGETARCH}"

###########################################################################
# PLAN
###########################################################################

plan:
  ARG TARGETARCH
  ARG TARGETVARIANT
  ARG MUSL
  FROM +rust

  COPY . .
  RUN cargo chef prepare --recipe-path recipe.json
  SAVE ARTIFACT recipe.json

###########################################################################
# DEPS
###########################################################################

deps:
  ARG TARGETARCH
  ARG TARGETVARIANT
  ARG MUSL
  FROM +rust

  COPY +plan/recipe.json .
  RUN cargo chef cook --recipe-path recipe.json --target ${target} --release --package ${package}
  SAVE IMAGE --push "ghcr.io/yolodev/${package}-build-cache:deps-${TARGETARCH}${TARGETVARIANT:+-${TARGETVARIANT}}${MUSL:+-musl}"

###########################################################################
# BUILD
###########################################################################

build:
  ARG TARGETARCH
  ARG TARGETVARIANT
  ARG MUSL
  FROM +deps

  COPY --dir . .
  RUN cargo build --target ${target} --release --package ${package} --locked --bin ${package}
  SAVE ARTIFACT target/${target}/release/${package}

###########################################################################
# VERSION HELPER
###########################################################################

version:
  FROM +rust

  COPY --dir . .
  RUN mkdir -p "/out" && cargo pkgid --package ${package} | cut -d# -f2 | cut -d: -f2 > /out/.version

  WORKDIR /out
  RUN echo "version=$(cat .version)"

  SAVE ARTIFACT /out/.version

###########################################################################
# ARTIFACTS
###########################################################################

crc:
  ARG TARGETARCH
  ARG TARGETVARIANT
  ARG MUSL
  FROM +version

  COPY +build/${package} /out/
  RUN mv ${package} "${package}-v$(cat .version)-${platform}"
  RUN sha256sum "${package}-v$(cat .version)-${platform}" > "${package}-v$(cat .version)-${platform}".sha256.txt
  RUN rm .version

  SAVE ARTIFACT /out/ /

artifacts:
  ARG TARGETARCH
  ARG TARGETVARIANT
  ARG MUSL
  FROM scratch

  COPY --platform=${USERPLATFORM} +crc/ /out/
  SAVE ARTIFACT /out/*

###########################################################################
# GROUP TARGETS
###########################################################################

amd64:
  COPY (+artifacts/* --TARGETPLATFORM=linux/amd64 --TARGETARCH=amd64 --TARGETVARIANT= --MUSL=) /out/
  COPY (+artifacts/* --TARGETPLATFORM=linux/amd64 --TARGETARCH=amd64 --TARGETVARIANT= --MUSL=1) /out/

  SAVE ARTIFACT /out/*

i686:
  COPY (+artifacts/* --TARGETPLATFORM=linux/i686 --TARGETARCH=i686 --TARGETVARIANT= --MUSL=) /out/
  # TODO: Fix
  # COPY (+artifacts/* --TARGETPLATFORM=linux/i686 --TARGETARCH=i686 --TARGETVARIANT= --MUSL=1) /out/

  SAVE ARTIFACT /out/*

arm64:
  COPY (+artifacts/* --TARGETPLATFORM=linux/arm64 --TARGETARCH=arm64 --TARGETVARIANT= --MUSL=) /out/
  COPY (+artifacts/* --TARGETPLATFORM=linux/arm64 --TARGETARCH=arm64 --TARGETVARIANT= --MUSL=1) /out/

  SAVE ARTIFACT /out/*

arm-v6:
  COPY (+artifacts/* --TARGETPLATFORM=linux/arm/v6 --TARGETARCH=arm --TARGETVARIANT=v6 --MUSL=) /out/
  # TODO: Fix
  # COPY (+artifacts/* --TARGETPLATFORM=linux/arm/v6 --TARGETARCH=arm --TARGETVARIANT=v6 --MUSL=1) /out/

  SAVE ARTIFACT /out/*

arm-v7:
  COPY (+artifacts/* --TARGETPLATFORM=linux/arm/v7 --TARGETARCH=arm --TARGETVARIANT=v7 --MUSL=) /out/
  # TODO: Fix
  # COPY (+artifacts/* --TARGETPLATFORM=linux/arm/v7 --TARGETARCH=arm --TARGETVARIANT=v7 --MUSL=1) /out/

  SAVE ARTIFACT /out/*

all:
  COPY +amd64/* /out/
  COPY +i686/* /out/
  COPY +arm64/* /out/
  COPY +arm-v6/* /out/
  COPY +arm-v7/* /out/

  SAVE ARTIFACT /out/*

###########################################################################
# IMAGE TARGETS
###########################################################################

image:
  ARG TARGETPLATFORM
  ARG TARGETARCH
  ARG TARGETVARIANT
  ARG VERSION=0.0.0-dirty
  FROM --platform=$TARGETPLATFORM debian:stable-slim

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
    --build-arg TARGETPLATFORM=${TARGETPLATFORM} \
    --build-arg TARGETARCH=${TARGETARCH} \
    --build-arg TARGETVARIANT=${TARGETVARIANT} \
    --build-arg MUSL= \
    +build/${package} /usr/bin/${package}

  RUN chown $APP_USER:$APP_USER /usr/bin/${package}

  USER $APP_USER
  ENV APP=/usr/bin/${package}
  CMD ${APP}

  SAVE IMAGE --push ghcr.io/yolodev/${package}:latest
  SAVE IMAGE --push ghcr.io/yolodev/${package}:v${VERSION}

alpine-image:
  ARG TARGETPLATFORM
  ARG TARGETARCH
  ARG TARGETVARIANT
  ARG VERSION=0.0.0-dirty
  FROM --platform=$TARGETPLATFORM alpine

  RUN apk --no-cache add ca-certificates tzdata

  EXPOSE 8080
  ENV TZ=Etc/UTC
  ENV APP_USER=appuser

  RUN addgroup $APP_USER \
    && adduser --ingroup $APP_USER $APP_USER --disabled-password --no-create-home

  COPY \
    --platform=linux/amd64 \
    --build-arg TARGETPLATFORM=${TARGETPLATFORM} \
    --build-arg TARGETARCH=${TARGETARCH} \
    --build-arg TARGETVARIANT=${TARGETVARIANT} \
    --build-arg MUSL=1 \
    +build/${package} /usr/bin/${package}

  RUN chown $APP_USER:$APP_USER /usr/bin/${package}

  USER $APP_USER
  ENV APP=/usr/bin/${package}
  CMD ${APP}

  SAVE IMAGE --push ghcr.io/yolodev/${package}:alpine-latest
  SAVE IMAGE --push ghcr.io/yolodev/${package}:alpine-v${VERSION}

images:
  BUILD --platform=linux/amd64 --platform=linux/386 --platform=linux/arm64 --platform=linux/arm/v6 --platform=linux/arm/v7 +image
  BUILD --platform=linux/amd64 --platform=linux/arm64 +alpine-image
