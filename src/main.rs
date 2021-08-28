use std::{io, thread};
use std::cell::RefCell;
use std::convert::TryInto;
use std::fs::File;
use std::io::{BufRead, BufReader, Error, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::os::unix::prelude::AsRawFd;
use std::path::Path;
use std::process::exit;
use std::sync::{Arc, Mutex, RwLock};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::SystemTime;

use kv::{Bucket, Config, Msgpack, Raw, Store, Value};
use percent_encoding::percent_decode;
use ring::digest::{Context, Digest, SHA256};
use rustls::{AllowAnyAnonymousOrAuthenticatedClient, Certificate, Connection, RootCertStore, ServerConnection};
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

use crate::client_verifier::CustomClientAuth;

mod client_verifier;

#[derive(StructOpt, Debug)]
#[structopt()]
struct Args {
    /// Path to the pkcs12 keyfile used for TLS connections
    #[structopt()]
    keyfile_path: String,

    /// Password to the supplied pkcs12 keyfile
    #[structopt()]
    keyfile_password: String,

    /// Bind server to this address
    #[structopt(default_value = "0.0.0.0:1965", short, long)]
    address: SocketAddr,
}

#[derive(Serialize, Deserialize, Debug)]
struct User {
    name: String,
    health: u8,
}

fn load_certs(filename: &str) -> Vec<rustls::Certificate> {
    let certfile = File::open(filename).expect("cannot open certificate file");
    let mut reader = BufReader::new(certfile);
    rustls_pemfile::certs(&mut reader)
        .unwrap()
        .iter()
        .map(|v| rustls::Certificate(v.clone()))
        .collect()
}

fn load_private_key(filename: &str) -> rustls::PrivateKey {
    let keyfile = File::open(filename).expect("cannot open private key file");
    let mut reader = BufReader::new(keyfile);

    loop {
        match rustls_pemfile::read_one(&mut reader).expect("cannot parse private key .pem file") {
            Some(rustls_pemfile::Item::RSAKey(key)) => return rustls::PrivateKey(key),
            Some(rustls_pemfile::Item::PKCS8Key(key)) => return rustls::PrivateKey(key),
            None => break,
            _ => {}
        }
    }

    panic!(
        "no keys found in {:?} (encrypted keys not supported)",
        filename
    );
}

fn make_config(keyfile_path: String, certfile_path: String) -> Arc<rustls::ServerConfig> {
    let mut roots = RootCertStore::empty();
    let privkey = load_private_key(&keyfile_path);
    let certs = load_certs(&certfile_path);
    roots.add(&certs[0]);
    let client_auth = CustomClientAuth::new(roots);

    let mut config = rustls::ServerConfig::builder()
        .with_safe_defaults()
        .with_client_cert_verifier(client_auth)
        .with_single_cert(certs, privkey)
        .expect("bad certificates/private key");

    config.key_log = Arc::new(rustls::KeyLogFile::new());

    Arc::new(config)
}

const BANNER: &str = "```Landscape with a dragon and a sphinx
           ,   ,
         ,-`{-`/
      ,-~ , \\ {-~~-,
    ,~  ,   ,`,-~~-,`,
  ,`   ,   { {      } }                                             }/
 ;     ,--/`\\ \\    / /                                     }/      /,/
;  ,-./      \\ \\  { {  (                                  /,;    ,/ ,/
; /   `       } } `, `-`-.___                            / `,  ,/  `,/
 \\|         ,`,`    `~.___,---}                         / ,`,,/  ,`,;
  `        { {                                     __  /  ,`/   ,`,;
        /   \\ \\                                 _,`, `{  `,{   `,`;`
       {     } }       /~\\         .-:::-.     (--,   ;\\ `,}  `,`;
       \\\\._./ /      /` , \\      ,:::::::::,     `~;   \\},/  `,`;     ,-=-
        `-..-`      /. `  .\\_   ;:::::::::::;  __,{     `/  `,`;     {
                   / , ~ . ^ `~`\\:::::::::::<<~>-,,`,    `-,  ``,_    }
                /~~ . `  . ~  , .`~~\\:::::::;    _-~  ;__,        `,-`
       /`\\    /~,  . ~ , '  `  ,  .` \\::::;`   <<<~```   ``-,,__   ;
      /` .`\\ /` .  ^  ,  ~  ,  . ` . ~\\~                       \\\\, `,__
     / ` , ,`\\.  ` ~  ,  ^ ,  `  ~ . . ``~~~`,                   `-`--, \\
    / , ~ . ~ \\ , ` .  ^  `  , . ^   .   , ` .`-,___,---,__            ``
  /` ` . ~ . ` `\\ `  ~  ,  .  ,  `  ,  . ~  ^  ,  .  ~  , .`~---,___
/` . `  ,  . ~ , \\  `  ~  ,  .  ^  ,  ~  .  `  ,  ~  .  ^  ,  ~  .  `-,
```";

fn main() {
    let Args { address, keyfile_path, keyfile_password } = Args::from_args();
    let listener = TcpListener::bind(address).unwrap();
    let fd = listener.as_raw_fd();
    let tls_config = make_config(keyfile_path, keyfile_password);
    // Configure the database
    let mut cfg = Config::new("database");
    let store = Arc::new(Mutex::new(Store::new(cfg).unwrap()));

    loop {
        match listener.accept() {
            Ok((mut stream, address)) => {
                eprintln!("New connection: {}", address);
                let tls_config = Arc::clone(&tls_config);
                let store = store.clone();
                thread::spawn(move || {
                    match rustls::ServerConnection::new(tls_config) {
                        Ok(mut tls_conn) => {
                            while tls_conn.is_handshaking() {
                                eprintln!("We handshaking {}", tls_conn.is_handshaking());
                                eprintln!("We reading {}", tls_conn.wants_read());
                                eprintln!("We writing {}", tls_conn.wants_write());
                                eprintln!();

                                if tls_conn.wants_read() {
                                    eprintln!("Reading...");
                                    match tls_conn.read_tls(&mut stream) {
                                        Ok(0) => {
                                            todo!("Conn closed");
                                        }
                                        Ok(count) => {
                                            eprintln!("Count {}", count);
                                            let io_state = tls_conn.process_new_packets().unwrap();
                                            eprintln!("{:?}", io_state);
                                        }
                                        Err(e) if e.kind() == io::ErrorKind::WouldBlock => {}
                                        Err(e) => panic!(e),
                                    };
                                    eprintln!("Read");
                                }

                                if tls_conn.wants_write() {
                                    eprintln!("Writing...");
                                    tls_conn.write_tls(&mut stream).unwrap();
                                    eprintln!("Wrote");
                                }
                            }

                            match tls_conn.peer_certificates() {
                                Some(peer_certificates) => {
                                    match peer_certificates.last() {
                                        Some(peer_certificate) => {
                                            let fingerprint = ring::digest::digest(&SHA256, peer_certificate.0.as_slice());
                                            tls_conn.process_new_packets().unwrap();
                                            let mut buffer = [0; 1024 + 2];
                                            let count = tls_conn.reader().read(&mut buffer).unwrap();
                                            match std::str::from_utf8(&buffer[0..count]) {
                                                Ok(request) => {
                                                    let (url, _) = request.split_once("\r\n").unwrap();
                                                    let url = url::Url::parse(url).unwrap();
                                                    eprintln!("Request: {}", url);
                                                    eprintln!("Request-path: {}", url.path());
                                                    eprintln!("Request-query: {:?}", url.query_pairs().map(|(k, v)| { format!("{}: {}", k, v) }).collect::<Vec<String>>());
                                                    let whole_query = url.query().map(|x| percent_decode(x.as_bytes()).decode_utf8().unwrap());
                                                    eprintln!("Request-query: {:?}", whole_query);

                                                    fn serve_frontpage(mut tls_conn: ServerConnection, mut stream: TcpStream, store: Arc<Mutex<Store>>, fingerprint: Digest) {
                                                        let store = store.lock().unwrap();
                                                        let bucket = store.bucket::<Raw, Msgpack<User>>(None).unwrap();
                                                        let result = bucket.get(fingerprint.as_ref());
                                                        let user = match result.unwrap() {
                                                            Some(data) => data.0,
                                                            None => User { name: "Alien".to_string(), health: 10 },
                                                        };
                                                        let msg = format!("20 text/gemini; lang=en\r\n{}\r\n### 🐉 Hello, {}!\r\nHP: {}\r\n=> set-name Set name\r\n### Actions\r\n=> fight ⚔ Fight\r\n=> rest 🏥 Rest", BANNER, user.name, user.health).into_bytes();
                                                        tls_conn.writer().write_all(&msg).unwrap();
                                                        tls_conn.send_close_notify();
                                                        tls_conn.write_tls(&mut stream).unwrap();
                                                    }

                                                    fn redirect(mut tls_conn: ServerConnection, mut stream: TcpStream, store: Arc<Mutex<Store>>, fingerprint: Digest) {
                                                        tls_conn.writer().write_all(b"30 /\r\n").unwrap();
                                                        tls_conn.send_close_notify();
                                                        tls_conn.write_tls(&mut stream).unwrap();
                                                    }

                                                    match url.path() {
                                                        "/" | "" => {
                                                            eprintln!("Serving root");
                                                            serve_frontpage(tls_conn, stream, store, fingerprint);
                                                        }
                                                        "/set-name" => {
                                                            eprintln!("Serving set-name");
                                                            let name: Option<String> = url.query().and_then(|x| percent_decode(x.as_bytes()).decode_utf8().map(|x| x.into()).ok());
                                                            match name {
                                                                None => {
                                                                    eprintln!("No name :(");
                                                                    let msg = b"10 Set name\r\n".as_ref();
                                                                    tls_conn.writer().write_all(&msg).unwrap();
                                                                    tls_conn.send_close_notify();
                                                                    tls_conn.write_tls(&mut stream).unwrap();
                                                                }
                                                                Some(name) => {
                                                                    eprintln!("We got a name :), {}", name);
                                                                    {
                                                                        let store = store.lock().unwrap();
                                                                        let bucket = store.bucket::<Raw, Msgpack<User>>(None).unwrap();
                                                                        let result = bucket.get(fingerprint.as_ref());
                                                                        let user = match result.unwrap() {
                                                                            Some(data) => data.0,
                                                                            None => User { name: "Alien".to_string(), health: 10 },
                                                                        };

                                                                        bucket.set(fingerprint.as_ref(), Msgpack(User { name, ..user }));
                                                                    }

                                                                    redirect(tls_conn, stream, store, fingerprint);
                                                                }
                                                            }
                                                        }
                                                        "/fight" => {
                                                            {
                                                                let store = store.lock().unwrap();
                                                                let bucket = store.bucket::<Raw, Msgpack<User>>(None).unwrap();
                                                                let result = bucket.get(fingerprint.as_ref());
                                                                let user = match result.unwrap() {
                                                                    Some(data) => data.0,
                                                                    None => User { name: "Alien".to_string(), health: 10 },
                                                                };

                                                                bucket.set(fingerprint.as_ref(), Msgpack(User { health: user.health - 1, ..user }));
                                                            }

                                                            redirect(tls_conn, stream, store, fingerprint);
                                                        }
                                                        "/rest" => {
                                                            {
                                                                let store = store.lock().unwrap();
                                                                let bucket = store.bucket::<Raw, Msgpack<User>>(None).unwrap();
                                                                let result = bucket.get(fingerprint.as_ref());
                                                                let user = match result.unwrap() {
                                                                    Some(data) => data.0,
                                                                    None => User { name: "Alien".to_string(), health: 10 },
                                                                };

                                                                bucket.set(fingerprint.as_ref(), Msgpack(User { health: user.health + 1, ..user }));
                                                            }
                                                            redirect(tls_conn, stream, store, fingerprint);
                                                        }
                                                        _ => {
                                                            let msg = b"51".as_ref();
                                                            tls_conn.writer().write_all(&msg).unwrap();
                                                            tls_conn.send_close_notify();
                                                            tls_conn.write_tls(&mut stream).unwrap();
                                                        }
                                                    }
                                                }
                                                Err(error) => {
                                                    eprintln!("Failed to parse request from {}: {}", address, error);
                                                    tls_conn.writer().write_all(b"59 Failed to parse utf8 string\r\n").unwrap();
                                                    tls_conn.write_tls(&mut stream).unwrap();
                                                    tls_conn.send_close_notify();
                                                }
                                            };
                                        }
                                        None => {
                                            tls_conn.writer().write_all(b"60\r\n").unwrap();
                                            tls_conn.write_tls(&mut stream).unwrap();
                                            tls_conn.send_close_notify();
                                        }
                                    }
                                }
                                None => {
                                    tls_conn.writer().write_all(b"60\r\n").unwrap();
                                    tls_conn.write_tls(&mut stream).unwrap();
                                    tls_conn.send_close_notify();
                                }
                            };
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
