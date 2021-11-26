VERSION 0.6

ARG USERPLATFORM=linux/amd64
ARG package=three-commas-scraper
ARG openssl=openssl-3.0.0

FROM rust
WORKDIR /src

RUN apt-get update && apt-get install --no-install-recommends -y rsync

###########################################################################
# MUSL COMPILER
###########################################################################

musl-compiler:
  ARG ARCH

  RUN apt-get update && apt-get install --no-install-recommends -y rsync

  # install musl compilers
  RUN curl -O "https://musl.cc/${ARCH}-linux-musl-cross.tgz" \
    && tar xzf "${ARCH}-linux-musl-cross.tgz" \
    && rm -f $(find "${ARCH}-linux-musl-cross" -name "ld-musl-*.so.1") \
    && rm "${ARCH}-linux-musl-cross/usr" \
    && rsync --ignore-errors -rLaq "${ARCH}-linux-musl-cross/" / || true

  SAVE ARTIFACT ${ARCH}-linux-musl-cross/ musl
  SAVE IMAGE --push ghcr.io/yolodev/rust-builder:musl-compiler-${ARCH}

###########################################################################
# OPENSSL
###########################################################################

openssl-src:
  RUN mkdir -p /musl/aarch64/ \
    && mkdir -p /musl/x86_64/ \
    && cd /tmp \
    && wget https://github.com/openssl/openssl/archive/${openssl}.tar.gz \
    && tar zxvf ${openssl}.tar.gz \
    && rm ${openssl}.tar.gz \
    && mv openssl-${openssl} openssl

  SAVE ARTIFACT /tmp/openssl
  SAVE IMAGE --push ghcr.io/yolodev/rust-builder:openssl-src

openssl-musl:
  FROM +musl-compiler
  ARG ARCH

  COPY +openssl-src/openssl /src/openssl

  RUN cd /src/openssl \
    && CC="${ARCH}-linux-musl-gcc -fPIE -pie" ./Configure no-shared no-async --prefix=/musl --openssldir=/musl linux-${ARCH} \
    && make depend > /dev/null \
    && make -j$(nproc) > /dev/null \
    && make install > /dev/null

  RUN if [ -d /musl/lib64 ] ; then mv /musl/lib64 /musl/lib ; fi

  SAVE ARTIFACT /musl/include /include
  SAVE ARTIFACT /musl/lib /lib
  SAVE IMAGE --push ghcr.io/yolodev/rust-builder:openssl-musl-${ARCH}

###########################################################################
# BASE
###########################################################################

base-amd64:
  ENV target=x86_64-unknown-linux-gnu
  ENV platform=amd64-linux-gnu

  RUN dpkg --add-architecture amd64
  RUN apt-get update && apt-get install --no-install-recommends -y rsync gcc-x86-64-linux-gnu libc6-dev-amd64-cross libssl-dev:amd64

  ENV CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=x86_64-linux-gnu-gcc
  ENV X86_64_UNKNOWN_LINUX_GNU_OPENSSL_LIB_DIR=/usr/lib/x86_64-linux-gnu/
  ENV X86_64_UNKNOWN_LINUX_GNU_OPENSSL_INCLUDE_DIR=/usr/include/openssl/

  SAVE IMAGE --push ghcr.io/yolodev/rust-builder:base-amd64

base-amd64-musl:
  ENV target=x86_64-unknown-linux-musl
  ENV platform=amd64-linux-musl

  # musl compiler
  COPY --build-arg ARCH=x86_64 +musl-compiler/musl /musl/x86_64
  RUN rsync --ignore-errors -rLaq /musl/x86_64/ /

  # musl openssl
  COPY --build-arg ARCH=x86_64 +openssl-musl/include /usr/include/x86_64-linux-musl
  COPY --build-arg ARCH=x86_64 +openssl-musl/lib /usr/lib/x86_64-linux-musl

  ENV CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER=x86_64-linux-musl-gcc
  ENV CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_RUSTFLAGS="-C target-feature=+crt-static -C link-self-contained=yes -Clinker=rust-lld"
  ENV CC_x86_64_unknown_linux_musl=x86_64-linux-musl-gcc
  ENV X86_64_UNKNOWN_LINUX_MUSL_OPENSSL_STATIC=true
  ENV X86_64_UNKNOWN_LINUX_MUSL_OPENSSL_LIB_DIR=/usr/lib/x86_64-linux-musl/
  ENV X86_64_UNKNOWN_LINUX_MUSL_OPENSSL_INCLUDE_DIR=/usr/include/x86_64-linux-musl/

  SAVE IMAGE --push ghcr.io/yolodev/rust-builder:base-amd64-musl

base-arm64:
  ENV target=aarch64-unknown-linux-gnu
  ENV platform=arm64-linux-gnu

  RUN dpkg --add-architecture arm64
  RUN apt-get update && apt-get install --no-install-recommends -y rsync gcc-aarch64-linux-gnu libc6-dev-arm64-cross libssl-dev:arm64

  ENV CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc
  ENV AARCH64_UNKNOWN_LINUX_GNU_OPENSSL_LIB_DIR=/usr/lib/aarch64-linux-gnu/
  ENV AARCH64_UNKNOWN_LINUX_GNU_OPENSSL_INCLUDE_DIR=/usr/include/openssl/

  SAVE IMAGE --push ghcr.io/yolodev/rust-builder:base-arm64

base-arm64-musl:
  ENV target=aarch64-unknown-linux-musl
  ENV platform=arm64-linux-musl

  # musl compiler
  COPY --build-arg ARCH=aarch64 +musl-compiler/musl /musl/aarch64
  RUN rsync --ignore-errors -rLaq /musl/aarch64/ /

  # musl openssl
  COPY --build-arg ARCH=aarch64 +openssl-musl/include /usr/include/aarch64-linux-musl
  COPY --build-arg ARCH=aarch64 +openssl-musl/lib /usr/lib/aarch64-linux-musl

  ENV CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_LINKER=aarch64-linux-musl-gcc
  ENV CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_RUSTFLAGS="-C target-feature=+crt-static -C link-self-contained=yes -Clinker=rust-lld"
  ENV CC_aarch64_unknown_linux_musl=aarch64-linux-musl-gcc
  ENV AARCH64_UNKNOWN_LINUX_MUSL_OPENSSL_STATIC=true
  ENV AARCH64_UNKNOWN_LINUX_MUSL_OPENSSL_LIB_DIR=/usr/lib/aarch64-linux-musl/
  ENV AARCH64_UNKNOWN_LINUX_MUSL_OPENSSL_INCLUDE_DIR=/usr/include/aarch64-linux-musl/

  SAVE IMAGE --push ghcr.io/yolodev/rust-builder:base-arm64-musl

###########################################################################
# CHEF
###########################################################################

chef:
  RUN cargo install cargo-chef
  RUN cp $(which cargo-chef) /cargo-chef

  SAVE ARTIFACT /cargo-chef
  SAVE IMAGE --push ghcr.io/yolodev/rust-builder:chef

###########################################################################
# RUST
###########################################################################

rust:
  ARG TARGETARCH
  ARG TARGETVARIANT
  ARG MUSL
  FROM "+base-${TARGETARCH}${MUSL:+-musl}"

  COPY rust-toolchain.toml .
  RUN rustup target add "${target}"

  COPY +chef/cargo-chef /usr/bin/cargo-chef
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
  SAVE IMAGE --push "ghcr.io/yolodev/${package}-build-cache:deps-${TARGETARCH}"

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

arm64:
  COPY (+artifacts/* --TARGETPLATFORM=linux/arm64 --TARGETARCH=arm64 --TARGETVARIANT= --MUSL=) /out/
  COPY (+artifacts/* --TARGETPLATFORM=linux/arm64 --TARGETARCH=arm64 --TARGETVARIANT= --MUSL=1) /out/

  SAVE ARTIFACT /out/*

all:
  COPY +amd64/* /out/
  COPY +arm64/* /out/

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
  BUILD --platform=linux/amd64 --platform=linux/arm64 +image
  BUILD --platform=linux/amd64 --platform=linux/arm64 +alpine-image
