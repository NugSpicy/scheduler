FROM rust:1.69-alpine AS rust

WORKDIR "/usr/src/scheduler"
RUN apk add build-base
RUN apk add protoc
COPY . .

FROM rust AS release
RUN --mount=type=cache,target=/usr/local/cargo/registry \
  cargo build --release &&                            \
  cp ./target/release/scheduler /usr/local/bin

FROM rust AS dev
RUN --mount=type=cache,target=/usr/local/cargo/registry \
  --mount=type=cache,target=/.cache/target/debug      \
  cargo build --target-dir='/.cache/target' &&        \
  cp /.cache/target/debug/scheduler /usr/local/bin

CMD ["scheduler"]