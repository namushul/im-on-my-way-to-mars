use std::net::SocketAddr;

use structopt::StructOpt;

mod application;
mod gemini;
mod response;
mod storage;
mod duration;

#[derive(StructOpt, Debug)]
#[structopt()]
pub struct Args {
    /// Path to a certificate file encoded in PEM format for TLS connections
    #[structopt()]
    certificates_path: String,

    /// Path to a PKCS 8 or RSA keyfile encoded in PEM format for TLS connections
    #[structopt()]
    private_key_path: String,

    /// Bind server to this address
    #[structopt(default_value = "0.0.0.0:1965", short, long)]
    address: SocketAddr,
}

fn main() {
    eprintln!("Starting...");
    gemini::run(Args::from_args());
}
