FROM rust:latest
WORKDIR /usr/src/namushul
EXPOSE 1965
COPY Cargo.toml .
COPY Cargo.lock .
COPY src ./src
RUN cargo install --release --path .
CMD ["namushul", "cert.pem.decrypted", "cert.cert"]