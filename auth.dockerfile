FROM rust:bullseye as chef
RUN cargo install cargo-chef
WORKDIR app

FROM chef as planner
COPY auth .
COPY .env .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef as builder
RUN apt-get update && \
    apt-get install -y protobuf-compiler
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY auth .
RUN cargo build --release --bin auth

FROM debian:bullseye-slim
WORKDIR app
RUN apt-get update && \
    apt-get install -y protobuf-compiler
COPY .env .
COPY --from=builder /app/target/release/auth /app/auth
EXPOSE 50051
CMD ["/app/auth"]
