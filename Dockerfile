FROM rust:latest
WORKDIR /usr/src/namushul
EXPOSE 1965
COPY Cargo.toml .
COPY Cargo.lock .

# Create a dummy file so that we can run "cargo build" to build dependencies.
RUN mkdir src
RUN echo "fn main() {}" > src/main.rs
RUN cargo build --release

COPY src ./src
RUN cargo install --path .
CMD ["namushul", "/run/secrets/namushul_certificate.pem", "/run/secrets/namushul_private_key.pem"]