FROM rust:bullseye as chef
RUN cargo install cargo-chef
WORKDIR app

FROM chef as planner
COPY graphql-core .
COPY .env ./.env
RUN cargo chef prepare --recipe-path recipe.json

FROM chef as builder
RUN apt-get update && \
    apt-get install -y protobuf-compiler
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY graphql-core .
RUN cargo build --release --bin graphql-core

FROM debian:bullseye-slim
WORKDIR app
RUN apt-get update && \
    apt-get install -y protobuf-compiler
COPY .env ./.env
COPY --from=builder /app/target/release/graphql-core /app/graphql-core
EXPOSE 50051
CMD ["/app/graphql-core"]
