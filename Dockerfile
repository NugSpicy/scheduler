FROM rust:1.67

WORKDIR "/usr/src/scheduler"
RUN apt-get install libssl-dev
COPY . .

RUN --mount=type=cache,target=/usr/local/cargo/registry \
  --mount=type=cache,target=/.cache/target/debug      \
  cargo build --target-dir='/.cache/target' &&        \
  cp /.cache/target/debug/scheduler /usr/local/bin

CMD ["scheduler"]