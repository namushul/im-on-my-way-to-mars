use std::{io, thread};
use std::fs::File;
use std::io::{BufReader, Read, Write};
use std::net::TcpListener;
use std::sync::Arc;

use percent_encoding::percent_decode;
use ring::digest::SHA256;
use rustls::Connection;
use structopt::StructOpt;

use crate::Args;
use crate::client_verifier::AcceptAnyClientCert;
use crate::response::Response;
use crate::server::{Request, Server};

fn load_certificates(path: &str) -> Vec<rustls::Certificate> {
    let file = File::open(path).expect("Cannot open certificate file");
    let mut reader = BufReader::new(file);
    rustls_pemfile::certs(&mut reader)
        .expect("Cannot parse certificate PEM file")
        .iter()
        .map(|v| rustls::Certificate(v.clone()))
        .collect()
}

fn load_private_key(path: &str) -> rustls::PrivateKey {
    let file = File::open(path).expect("Cannot open private key file");
    let mut reader = BufReader::new(file);
    loop {
        match rustls_pemfile::read_one(&mut reader).expect("Cannot parse private key PEM file") {
            Some(rustls_pemfile::Item::RSAKey(key)) => return rustls::PrivateKey(key),
            Some(rustls_pemfile::Item::PKCS8Key(key)) => return rustls::PrivateKey(key),
            None => panic!("No keys found in {:?} (encrypted keys not supported)", path),
            _ => {}
        }
    }
}

fn make_config(private_key_path: String, certificate_path: String) -> Arc<rustls::ServerConfig> {
    Arc::new(rustls::ServerConfig::builder()
        .with_safe_defaults()
        .with_client_cert_verifier(AcceptAnyClientCert::new())
        .with_single_cert(load_certificates(&certificate_path), load_private_key(&private_key_path))
        .expect("Bad certificates/private key"))
}

pub fn run() {
    let Args { address, private_key_path, certificate_path } = Args::from_args();
    let listener = TcpListener::bind(address).unwrap();
    let tls_config = make_config(private_key_path, certificate_path);

    loop {
        match listener.accept() {
            Ok((mut stream, address)) => {
                eprintln!("New connection: {}", address);
                let tls_config = Arc::clone(&tls_config);
                thread::spawn(move || {
                    let server = Server {};
                    match rustls::ServerConnection::new(tls_config) {
                        Ok(mut tls_conn) => {
                            eprintln!("Performing handshake");
                            while tls_conn.is_handshaking() {
                                if tls_conn.wants_read() {
                                    eprintln!("Reading...");
                                    match tls_conn.read_tls(&mut stream) {
                                        Ok(0) => {
                                            panic!("Connection closed while handshaking");
                                        }
                                        Ok(count) => {
                                            eprintln!("Read {} bytes", count);
                                            let io_state = tls_conn.process_new_packets().unwrap();
                                            eprintln!("{:?}", io_state);
                                        }
                                        Err(e) if e.kind() == io::ErrorKind::WouldBlock => {}
                                        Err(e) => panic!("{}", e),
                                    };
                                }

                                if tls_conn.wants_write() {
                                    eprintln!("Writing...");
                                    match tls_conn.write_tls(&mut stream) {
                                        Ok(count) => eprintln!("Wrote {} bytes", count),
                                        Err(e) if e.kind() == io::ErrorKind::WouldBlock => {}
                                        Err(e) => panic!("{}", e),
                                    };
                                }
                            }
                            eprintln!("Handshake successful");

                            while tls_conn.wants_read() {
                                eprintln!("Reading...");
                                match tls_conn.read_tls(&mut stream) {
                                    Ok(0) => {}
                                    Ok(count) => {
                                        eprintln!("Read {} bytes", count);
                                        let io_state = tls_conn.process_new_packets().unwrap();
                                        eprintln!("{:?}", io_state);
                                    }
                                    Err(e) if e.kind() == io::ErrorKind::WouldBlock => {}
                                    Err(e) => panic!("{}", e),
                                };
                            }

                            let response = match tls_conn.peer_certificates() {
                                Some(peer_certificates) => {
                                    match peer_certificates.last() {
                                        Some(peer_certificate) => {
                                            let fingerprint = ring::digest::digest(&SHA256, peer_certificate.0.as_slice());
                                            tls_conn.process_new_packets().unwrap();
                                            let mut buffer = [0; 1024 + 2];
                                            let mut count = 0;
                                            while count == 0 {
                                                count = match tls_conn.reader().read(&mut buffer) {
                                                    Ok(count) => count,
                                                    Err(e) if e.kind() == io::ErrorKind::WouldBlock => 0,
                                                    Err(e) => panic!("{}", e),
                                                };
                                            }
                                            match std::str::from_utf8(&buffer[0..count]) {
                                                Ok(request) => {
                                                    match request.split_once("\r\n") {
                                                        Some((url, _)) => {
                                                            match url::Url::parse(url) {
                                                                Ok(url) => {
                                                                    eprintln!("Request: {}", url);
                                                                    eprintln!("Request-path: {}", url.path());
                                                                    eprintln!("Request-query: {:?}", url.query_pairs().map(|(k, v)| { format!("{}: {}", k, v) }).collect::<Vec<String>>());
                                                                    let whole_query = url.query().map(|x| percent_decode(x.as_bytes()).decode_utf8().unwrap().into());
                                                                    eprintln!("Request-query: {:?}", whole_query);

                                                                    let request = Request {
                                                                        url,
                                                                        query: whole_query,
                                                                        user_fingerprint: fingerprint,
                                                                    };
                                                                    server.handle_request(request)
                                                                }
                                                                Err(_) => Response::bad_request("Failed to parse url in request".to_string()),
                                                            }
                                                        }
                                                        None => Response::bad_request("Failed to parse request, expected a single \\r\\n to be present".to_string()),
                                                    }
                                                }
                                                Err(_) => Response::bad_request("Failed to parse utf8 string".to_string()),
                                            }
                                        }
                                        None => Response::client_certificate_required("".to_string()),
                                    }
                                }
                                None => Response::client_certificate_required("".to_string()),
                            };

                            tls_conn.writer().write_all(response.as_bytes()).unwrap();
                            tls_conn.send_close_notify();
                            tls_conn.write_tls(&mut stream).unwrap();
                        }
                        Err(error) => {
                            eprintln!("Failed to establish TLS connection to {}: {}", address, error);
                        }
                    };

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