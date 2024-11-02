FROM rust:slim

WORKDIR /usr/src/app

COPY . .

RUN cargo build

# Set the environment variable RUST_LOG
ENV RUST_LOG="info"

# Run the application with cargo
CMD ["cargo", "run", "--quiet"]