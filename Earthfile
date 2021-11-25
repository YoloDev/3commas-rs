ARG package=three-commas-scraper
ARG openssl=openssl-3.0.0

FROM busybox

###########################################################################
# MUSL DEPENDENCIES
###########################################################################

musl:
  FROM rust
  ARG ARCH

  RUN apt-get update && apt-get install --no-install-recommends -y rsync

  # install musl compilers
  RUN curl -O "https://musl.cc/${ARCH}-linux-musl-cross.tgz" \
    && tar xzf "${ARCH}-linux-musl-cross.tgz" \
    && rm -f $(find "${ARCH}-linux-musl-cross" -name "ld-musl-*.so.1") \
    && rm "${ARCH}-linux-musl-cross/usr" \
    && rsync --ignore-errors -rLaq "${ARCH}-linux-musl-cross/" / || true

  SAVE ARTIFACT ${ARCH}-linux-musl-cross/ musl
  SAVE IMAGE --cache-hint

openssl-src:
  FROM rust

  RUN mkdir -p /musl/aarch64/ \
    && mkdir -p /musl/x86_64/ \
    && cd /tmp \
    && wget https://github.com/openssl/openssl/archive/${openssl}.tar.gz \
    && tar zxvf ${openssl}.tar.gz \
    && rm ${openssl}.tar.gz \
    && mv openssl-${openssl} openssl

  SAVE ARTIFACT /tmp/openssl
  SAVE IMAGE --cache-hint

openssl-musl:
  FROM +musl
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
  # SAVE IMAGE --push ghcr.io/yolodev/${package}:openssl-musl-${ARCH}-test
  SAVE IMAGE --cache-hint

###########################################################################
# BASE IMAGES
###########################################################################

rust:
  FROM rust

  WORKDIR /src

  RUN dpkg --add-architecture arm64
  RUN apt-get update && apt-get install --no-install-recommends -y rsync gcc-aarch64-linux-gnu libc6-dev-arm64-cross libssl-dev:arm64

  # musl compilers
  COPY --build-arg ARCH=x86_64 +musl/musl /musl/x86_64
  COPY --build-arg ARCH=aarch64 +musl/musl /musl/aarch64
  RUN rsync --ignore-errors -rLaq /musl/x86_64/ / \
    && rsync --ignore-errors -rLaq /musl/aarch64/ /

  # musl openssl
  COPY --build-arg ARCH=x86_64 +openssl-musl/include /usr/include/x86_64-linux-musl
  COPY --build-arg ARCH=aarch64 +openssl-musl/include /usr/include/aarch64-linux-musl

  COPY --build-arg ARCH=x86_64 +openssl-musl/lib /usr/lib/x86_64-linux-musl
  COPY --build-arg ARCH=aarch64 +openssl-musl/lib /usr/lib/aarch64-linux-musl

  # install rust targets
  COPY rust-toolchain.toml .

  # gnu targets
  RUN rustup target add x86_64-unknown-linux-gnu
  RUN rustup target add aarch64-unknown-linux-gnu

  # musl targets
  RUN rustup target add x86_64-unknown-linux-musl
  RUN rustup target add aarch64-unknown-linux-musl

  # setup cargo env variables
  # gnu
  ENV CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc
  ENV AARCH64_UNKNOWN_LINUX_GNU_OPENSSL_LIB_DIR=/usr/lib/aarch64-linux-gnu/
  ENV AARCH64_UNKNOWN_LINUX_GNU_OPENSSL_INCLUDE_DIR=/usr/include/openssl/

  # musl
  ENV CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER=x86_64-linux-musl-gcc
  ENV CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_RUSTFLAGS="-C target-feature=+crt-static -C link-self-contained=yes -Clinker=rust-lld"
  ENV CC_x86_64_unknown_linux_musl=x86_64-linux-musl-gcc
  ENV X86_64_UNKNOWN_LINUX_MUSL_OPENSSL_STATIC=true
  ENV X86_64_UNKNOWN_LINUX_MUSL_OPENSSL_LIB_DIR=/usr/lib/x86_64-linux-musl/
  ENV X86_64_UNKNOWN_LINUX_MUSL_OPENSSL_INCLUDE_DIR=/usr/include/x86_64-linux-musl/

  ENV CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_LINKER=aarch64-linux-musl-gcc
  ENV CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_RUSTFLAGS="-C target-feature=+crt-static -C link-self-contained=yes -Clinker=rust-lld"
  ENV CC_aarch64_unknown_linux_musl=aarch64-linux-musl-gcc
  ENV AARCH64_UNKNOWN_LINUX_MUSL_OPENSSL_STATIC=true
  ENV AARCH64_UNKNOWN_LINUX_MUSL_OPENSSL_LIB_DIR=/usr/lib/aarch64-linux-musl/
  ENV AARCH64_UNKNOWN_LINUX_MUSL_OPENSSL_INCLUDE_DIR=/usr/include/aarch64-linux-musl/

  # SAVE IMAGE --push ghcr.io/yolodev/${package}:rust-test
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

amd64-linux-musl:
  FROM +version
  ENV target=x86_64-unknown-linux-musl
  ENV platform=amd64-linux-musl

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

arm64-linux-musl:
  FROM +version
  ENV target=aarch64-unknown-linux-musl
  ENV platform=aarch64-linux-musl

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
  COPY +amd64-linux-musl/* /out/

  SAVE ARTIFACT /out/*

arm64:
  COPY +arm64-linux-gnu/* /out/
  COPY +arm64-linux-musl/* /out/

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

build-image-musl:
  ARG TARGETPLATFORM
  ARG TARGETVARIANT
  ARG TARGETARCH
  ARG VERSION
  FROM --platform=$TARGETPLATFORM alpine

  IF [ "$TARGETARCH" = "amd64" ]
    ENV TARGET=x86_64-unknown-linux-musl
  ELSE
    ENV TARGET=aarch64-unknown-linux-musl
  END

  RUN apk --no-cache add ca-certificates tzdata

  EXPOSE 8080
  ENV TZ=Etc/UTC
  ENV APP_USER=appuser

  RUN addgroup $APP_USER \
    && adduser --ingroup $APP_USER $APP_USER --disabled-password --no-create-home

  COPY \
    --platform=linux/amd64 \
    --build-arg target=${TARGET} \
    +build/${package} /usr/bin/${package}-${TARGETARCH}

  RUN chown $APP_USER:$APP_USER /usr/bin/${package}-${TARGETARCH}

  USER $APP_USER
  ENV APP=/usr/bin/${package}-${TARGETARCH}
  CMD ${APP}

  SAVE IMAGE --push ghcr.io/yolodev/${package}:alpine-latest
  SAVE IMAGE --push ghcr.io/yolodev/${package}:alpine-v${VERSION}

image:
  BUILD --platform=linux/amd64 --platform=linux/arm64 +build-image
  BUILD --platform=linux/amd64 --platform=linux/arm64 +build-image-musl
