FROM rustlang/rust:nightly as builder

COPY . /src
WORKDIR /src
RUN --mount=type=cache,target=/src/target --mount=type=cache,target=$CARGO_HOME cargo build --package three-commas-scraper --release && mv target/release/three-commas-scraper /src/three-commas-scraper
# RUN --mount=type=cache,target=/src/target ls -la target && ls -la target/release && exit 1

FROM debian:buster-slim
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

COPY --from=builder /src/three-commas-scraper ${APP}/three-commas-scraper
RUN chown -R $APP_USER:$APP_USER ${APP}

USER $APP_USER
WORKDIR ${APP}

CMD ["./three-commas-scraper"]
