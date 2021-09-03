use std::net::SocketAddr;
use structopt::StructOpt;

mod client_verifier;
mod server;
mod gemini;
mod banner;
mod response;
mod storage;

#[derive(StructOpt, Debug)]
#[structopt()]
struct Args {
    /// Path to the pkcs12 keyfile used for TLS connections
    #[structopt()]
    keyfile_path: String,

    /// Path to the certificate file used for TLS connections
    #[structopt()]
    certfile_path: String,

    /// Bind server to this address
    #[structopt(default_value = "0.0.0.0:1965", short, long)]
    address: SocketAddr,
}

fn main() {
    eprintln!("Starting...");
    gemini::run();
}
