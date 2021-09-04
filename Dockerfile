FROM rust:latest
WORKDIR /usr/src/im-on-my-way-to-mars
EXPOSE 1965
COPY development_private_key.pem .
COPY development_private_key.pem .
COPY Cargo.toml .
COPY Cargo.lock .

# Create a dummy file so that we can run "cargo build" to build dependencies.
RUN mkdir src
RUN echo "fn main() {}" > src/main.rs
RUN cargo build

COPY src ./src
RUN cargo install --path .
CMD ["im-on-my-way-to-mars", "cert.pem.decrypted", "cert.cert"]