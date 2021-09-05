//! Handles running a gemini server.
//! This mostly consists of handling the TCP listener and the TLS.
//! The gemini part only makes up a small part in comparison.

use std::convert::TryInto;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::ops::Deref;
use std::sync::Arc;
use std::thread;

use openssl::hash::MessageDigest;
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod, SslStream, SslVerifyMode, SslVersion};
use percent_encoding::percent_decode;
use url::Url;

use crate::application::{Request, Application};
use crate::Args;
use crate::response::Response;
use std::time::Instant;

fn make_acceptor(private_key_path: String, certificates_path: String) -> Arc<SslAcceptor> {
    let mut acceptor = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    acceptor.set_min_proto_version(Some(SslVersion::TLS1_2)).unwrap();
    acceptor.set_verify_callback(SslVerifyMode::PEER, |_ver, store| {
        match store.error().as_raw() {
            18 => true, // Certificate self-signed
            _ => false
        }
    });
    acceptor.set_private_key_file(private_key_path, SslFiletype::PEM).unwrap();
    acceptor.set_certificate_chain_file(certificates_path).unwrap();
    acceptor.check_private_key().unwrap();
    Arc::new(acceptor.build())
}

pub fn read_request(stream: &mut SslStream<TcpStream>) -> Result<Url, Response> {
    const MAX_URL_LENGTH: usize = 1024;
    const CRLF_LENGTH: usize = 2;
    const MAX_REQUEST_LENGTH: usize = MAX_URL_LENGTH + CRLF_LENGTH;
    let mut buffer = [0; MAX_REQUEST_LENGTH];

    let mut total_count = stream.read(&mut buffer).unwrap();
    let mut closed = total_count == 0;
    loop {
        let ends_with_crlf = total_count >= 2 && buffer[total_count - 2] == b'\r' && buffer[total_count - 1] == b'\n';
        if ends_with_crlf {
            let request = std::str::from_utf8(&buffer[0..total_count])
                .or(Err(Response::bad_request("Failed to parse utf8 string".to_owned())))?;

            let url_string = request.split_once("\r\n")
                .map(|pair| pair.0)
                .ok_or(Response::bad_request("Failed to parse utf8 string".to_owned()))?;

            let url = url::Url::parse(url_string)
                .or(Err(Response::bad_request("Failed to parse url in request".to_owned())))?;

            return Ok(url);
        }

        if total_count == buffer.len() {
            return Err(Response::bad_request("Request too large".to_owned()));
        }

        if closed {
            return Err(Response::bad_request("Failed to parse request, expected \\r\\n".to_owned()));
        }

        let count = stream.read(&mut buffer[total_count..]).unwrap();
        closed = count == 0;
        total_count += count;
    }
}

pub fn handle_connection(stream: &mut SslStream<TcpStream>, start_time: Instant) -> Response {
    let server = Application::new(start_time);
    match read_request(stream) {
        Ok(url) => {
            let peer_fingerprint: Option<[u8; 32]> = match stream.ssl().peer_certificate() {
                Some(peer_certificate) => {
                    match peer_certificate.digest(MessageDigest::sha256()) {
                        Ok(peer_fingerprint) => Some(peer_fingerprint.deref().try_into().unwrap()),
                        Err(_) => return Response::temporary_failure("Failed to calculate digest of client certificate".to_owned())
                    }
                }
                None => None
            };
            let query = match url.query() {
                Some(query) => {
                    match percent_decode(query.as_bytes()).decode_utf8() {
                        Ok(query) => Some(query.into()),
                        Err(_) => return Response::bad_request("Query string contains invalid utf8".to_owned())
                    }
                }
                None => None
            };
            let request = Request { url, query, peer_fingerprint };
            server.handle_request(request)
        }
        Err(response) => response
    }
}

pub fn run(args: Args) {
    let Args { address, private_key_path, certificates_path } = args;
    let listener = TcpListener::bind(address).unwrap();
    let acceptor = make_acceptor(private_key_path, certificates_path);
    let start_time = Instant::now();

    loop {
        match listener.accept() {
            Ok((stream, address)) => {
                eprintln!("New connection: {}", address);
                let acceptor = acceptor.clone();
                thread::spawn(move || {
                    let mut stream = acceptor.accept(stream).unwrap();
                    let response = handle_connection(&mut stream, start_time);
                    stream.write_all(response.as_bytes()).unwrap();
                    stream.shutdown().unwrap();

                    eprintln!("Done with connection");
                });
            }
            Err(error) => {
                eprintln!("Error accepting connection: {}", error);
                break;
            }
        }
    }
}