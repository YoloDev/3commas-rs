VERSION 0.6

ARG USERPLATFORM=linux/amd64
ARG package=three-commas-scraper
ARG openssl=openssl-3.0.0

FROM scratch

###########################################################################
# ROOT
###########################################################################

root:
  FROM rust
  WORKDIR /src

  RUN apt-get update && apt-get install --no-install-recommends -y rsync

###########################################################################
# MUSL COMPILER
###########################################################################

musl-compiler:
  ARG ARCH
  ARG ABI=musl
  FROM +root

  RUN apt-get update && apt-get install --no-install-recommends -y rsync

  # install musl compilers
  RUN curl -O "https://musl.cc/${ARCH}-linux-${ABI}-cross.tgz" \
    && tar xzf "${ARCH}-linux-${ABI}-cross.tgz" \
    && rm -f $(find "${ARCH}-linux-${ABI}-cross" -name "ld-musl-*.so.1") \
    && rm "${ARCH}-linux-${ABI}-cross/usr" \
    && rsync --ignore-errors -rLaq "${ARCH}-linux-${ABI}-cross/" / || true

  SAVE ARTIFACT ${ARCH}-linux-${ABI}-cross/ musl
  SAVE IMAGE --push ghcr.io/yolodev/rust-builder:musl-compiler-${ARCH}-${ABI}

###########################################################################
# OPENSSL
###########################################################################

openssl-src:
  FROM +root
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
  ARG ABI=musl
  ARG OS=linux-${ARCH}

  COPY +openssl-src/openssl /src/openssl

  RUN cd /src/openssl \
    && CC="${ARCH}-linux-${ABI}-gcc -fPIE -pie" ./Configure no-shared no-async --prefix=/musl --openssldir=/musl ${OS} \
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
  FROM +root
  ENV target=x86_64-unknown-linux-gnu
  ENV platform=amd64-linux-gnu

  RUN dpkg --add-architecture amd64
  RUN apt-get update && apt-get install --no-install-recommends -y rsync gcc-x86-64-linux-gnu libc6-dev-amd64-cross libssl-dev:amd64

  ENV CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=x86_64-linux-gnu-gcc
  ENV X86_64_UNKNOWN_LINUX_GNU_OPENSSL_LIB_DIR=/usr/lib/x86_64-linux-gnu/
  ENV X86_64_UNKNOWN_LINUX_GNU_OPENSSL_INCLUDE_DIR=/usr/include/openssl/

  SAVE IMAGE --push ghcr.io/yolodev/rust-builder:base-amd64

base-amd64-musl:
  FROM +root
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

base-i686:
  FROM +root
  ENV target=i686-unknown-linux-gnu
  ENV platform=i686-linux-gnu

  RUN dpkg --add-architecture i386
  RUN apt-get update && apt-get install --no-install-recommends -y rsync gcc-i686-linux-gnu libc6-dev-i386-cross libssl-dev:i386

  ENV CARGO_TARGET_I686_UNKNOWN_LINUX_GNU_LINKER=i686-linux-gnu-gcc
  ENV I686_UNKNOWN_LINUX_GNU_OPENSSL_LIB_DIR=/usr/lib/i386-linux-gnu/
  ENV I686_UNKNOWN_LINUX_GNU_OPENSSL_INCLUDE_DIR=/usr/include/openssl/

  SAVE IMAGE --push ghcr.io/yolodev/rust-builder:base-i686

base-i686-musl:
  FROM +root
  ENV target=i686-unknown-linux-musl
  ENV platform=i686-linux-musl

  # musl compiler
  COPY --build-arg ARCH=i686 --build-arg OS=linux-x86 +musl-compiler/musl /musl/i686
  RUN rsync --ignore-errors -rLaq /musl/i686/ /

  # musl openssl
  COPY --build-arg ARCH=i686 --build-arg OS=linux-x86 +openssl-musl/include /usr/include/i686-linux-musl
  COPY --build-arg ARCH=i686 --build-arg OS=linux-x86 +openssl-musl/lib /usr/lib/i686-linux-musl

  ENV CARGO_TARGET_I686_UNKNOWN_LINUX_MUSL_LINKER=i686-linux-musl-gcc
  ENV CARGO_TARGET_I686_UNKNOWN_LINUX_MUSL_RUSTFLAGS="-C target-feature=+crt-static -C link-self-contained=yes -Clinker=rust-lld"
  ENV CC_i686_unknown_linux_musl=i686-linux-musl-gcc
  ENV I686_UNKNOWN_LINUX_MUSL_OPENSSL_STATIC=true
  ENV I686_UNKNOWN_LINUX_MUSL_OPENSSL_LIB_DIR=/usr/lib/i686-linux-musl/
  ENV I686_UNKNOWN_LINUX_MUSL_OPENSSL_INCLUDE_DIR=/usr/include/i686-linux-musl/

  SAVE IMAGE --push ghcr.io/yolodev/rust-builder:base-amd64-musl

base-arm64:
  FROM +root
  ENV target=aarch64-unknown-linux-gnu
  ENV platform=arm64-linux-gnu

  RUN dpkg --add-architecture arm64
  RUN apt-get update && apt-get install --no-install-recommends -y rsync gcc-aarch64-linux-gnu libc6-dev-arm64-cross libssl-dev:arm64

  ENV CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc
  ENV AARCH64_UNKNOWN_LINUX_GNU_OPENSSL_LIB_DIR=/usr/lib/aarch64-linux-gnu/
  ENV AARCH64_UNKNOWN_LINUX_GNU_OPENSSL_INCLUDE_DIR=/usr/include/openssl/

  SAVE IMAGE --push ghcr.io/yolodev/rust-builder:base-arm64

base-arm64-musl:
  FROM +root
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

base-arm-v6:
  FROM +root
  ENV target=arm-unknown-linux-gnueabi
  ENV platform=arm-linux-gnueabi

  RUN dpkg --add-architecture armel
  RUN apt-get update && apt-get install --no-install-recommends -y rsync gcc-arm-linux-gnueabi libc6-dev-armel-cross libssl-dev:armel

  ENV CARGO_TARGET_ARM_UNKNOWN_LINUX_GNUEABI_LINKER=arm-linux-gnueabi-gcc
  ENV ARM_UNKNOWN_LINUX_GNUEABI_OPENSSL_LIB_DIR=/usr/lib/arm-linux-gnueabi/
  ENV ARM_UNKNOWN_LINUX_GNUEABI_OPENSSL_INCLUDE_DIR=/usr/include/openssl/

  SAVE IMAGE --push ghcr.io/yolodev/rust-builder:base-arm-v6

base-arm-v6-musl:
  FROM +root
  ENV target=arm-unknown-linux-musleabi
  ENV platform=arm-linux-musleabi

  # musl compiler
  COPY --build-arg ARCH=arm --build-arg ABI=musleabi +musl-compiler/musl /musl/armel
  RUN rsync --ignore-errors -rLaq /musl/armel/ /

  # musl openssl
  COPY --build-arg ARCH=arm --build-arg ABI=musleabi --build-arg OS=linux-armv4 +openssl-musl/include /usr/include/arm-linux-musleabi
  COPY --build-arg ARCH=arm --build-arg ABI=musleabi --build-arg OS=linux-armv4 +openssl-musl/lib /usr/lib/arm-linux-musleabi
  # RUN cp /arm-linux-musleabi/lib/libatomic.* /usr/lib/arm-linux-musleabi/

  ENV CARGO_TARGET_ARM_UNKNOWN_LINUX_MUSLEABI_LINKER=arm-linux-musleabi-gcc
  ENV CARGO_TARGET_ARM_UNKNOWN_LINUX_MUSLEABI_RUSTFLAGS="-C target-feature=+crt-static -C link-self-contained=yes -Clinker=rust-lld"
  ENV CC_arm_unknown_linux_musleabi=arm-linux-musleabi-gcc
  ENV ARM_UNKNOWN_LINUX_MUSLEABI_OPENSSL_STATIC=true
  ENV ARM_UNKNOWN_LINUX_MUSLEABI_OPENSSL_LIB_DIR=/usr/lib/arm-linux-musleabi
  ENV ARM_UNKNOWN_LINUX_MUSLEABI_OPENSSL_INCLUDE_DIR=/usr/include/arm-linux-musleabi/

  SAVE IMAGE --push ghcr.io/yolodev/rust-builder:base-arm-v6-musl

base-arm-v7:
  FROM +root
  ENV target=armv7-unknown-linux-gnueabihf
  ENV platform=armv7-linux-gnueabihf

  RUN dpkg --add-architecture armhf
  RUN apt-get update && apt-get install --no-install-recommends -y rsync gcc-arm-linux-gnueabihf libc6-dev-armhf-cross libssl-dev:armhf

  ENV CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_LINKER=arm-linux-gnueabihf-gcc
  ENV ARMV7_UNKNOWN_LINUX_GNUEABIHF_OPENSSL_LIB_DIR=/usr/lib/arm-linux-gnueabihf/
  ENV ARMV7_UNKNOWN_LINUX_GNUEABIHF_OPENSSL_INCLUDE_DIR=/usr/include/openssl/

  SAVE IMAGE --push ghcr.io/yolodev/rust-builder:base-arm-v7

base-arm-v7-musl:
  FROM +root
  ENV target=armv7-unknown-linux-musleabihf
  ENV platform=armv7-linux-musleabihf

  # musl compiler
  COPY --build-arg ARCH=arm --build-arg ABI=musleabihf +musl-compiler/musl /musl/armhf
  RUN rsync --ignore-errors -rLaq /musl/armhf/ /

  # musl openssl
  COPY --build-arg ARCH=arm --build-arg ABI=musleabihf --build-arg OS=linux-armv4 +openssl-musl/include /usr/include/arm-linux-musleabihf
  COPY --build-arg ARCH=arm --build-arg ABI=musleabihf --build-arg OS=linux-armv4 +openssl-musl/lib /usr/lib/arm-linux-musleabihf
  # RUN cp /arm-linux-musleabi/lib/libatomic.* /usr/lib/arm-linux-musleabi/

  ENV CARGO_TARGET_ARMV7_UNKNOWN_LINUX_MUSLEABIHF_LINKER=arm-linux-musleabihf-gcc
  ENV CARGO_TARGET_ARMV7_UNKNOWN_LINUX_MUSLEABIHF_RUSTFLAGS="-C target-feature=+crt-static -C link-self-contained=yes -Clinker=rust-lld"
  ENV CC_armv7_unknown_linux_musleabi=arm-linux-musleabihf-gcc
  ENV ARMV7_UNKNOWN_LINUX_MUSLEABIHF_OPENSSL_STATIC=true
  ENV ARMV7_UNKNOWN_LINUX_MUSLEABIHF_OPENSSL_LIB_DIR=/usr/lib/arm-linux-musleabihf/
  ENV ARMV7_UNKNOWN_LINUX_MUSLEABIHF_OPENSSL_INCLUDE_DIR=/usr/include/arm-linux-musleabihf/

  SAVE IMAGE --push ghcr.io/yolodev/rust-builder:base-arm-v7-musl

###########################################################################
# BASE ALIASES
###########################################################################

base-i386:
  FROM +base-i686

base-i386-musl:
  FROM +base-i686-musl

base-386:
  FROM +base-i686

base-386-musl:
  FROM +base-i686-musl

###########################################################################
# CHEF
###########################################################################

chef:
  FROM +root
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
  FROM "+base-${TARGETARCH}${TARGETVARIANT:+-${TARGETVARIANT}}${MUSL:+-musl}"

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
