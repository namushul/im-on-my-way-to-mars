use url::Url;

use crate::response::Response;
use crate::storage::{Storage, User};

#[derive(Debug)]
pub struct Server {}

#[derive(Debug)]
pub struct Request {
    pub url: Url,
    pub query: Option<String>,
    pub peer_fingerprint: Option<[u8; 32]>,
}

const BANNER: &str = include_str!("banner.txt");

fn serve_frontpage(user: User) -> Response {
    Response::success("text/gemini; lang=en".to_string(), format!("{banner}\r\n### 🐉 Hello, {name}!\r\nHP: {health}/{max_health}\r\n=> set-name 📝 Set name\r\n### Actions\r\n=> fight ⚔ Fight\r\n=> rest 🏥 Rest", banner = BANNER, name = user.name, health = user.health, max_health = user.max_health))
}


impl Server {
    pub fn handle_request(&self, request: Request) -> Response {
        eprintln!("Request: {}", request.url);
        eprintln!("Request-path: {}", request.url.path());
        eprintln!("Request-query: {:?}", request.url.query_pairs().map(|(k, v)| { format!("{}: {}", k, v) }).collect::<Vec<String>>());
        eprintln!("Request-query: {:?}", request.query);

        let mut storage = match Storage::new() {
            Ok(storage) => storage,
            Err(_) => return Response::temporary_failure("Failed to connect to database".into()),
        };

        let fingerprint = &match request.peer_fingerprint {
            Some(f) => f,
            None => return Response::client_certificate_required("".to_string())
        };

        let user = match storage.get_or_create_user(fingerprint) {
            Ok(user) => user,
            Err(_) => return Response::temporary_failure("Failed to get/create user".into()),
        };
        eprintln!("User: {:?}", user);

        match request.url.path() {
            "/" | "" => {
                serve_frontpage(user)
            }
            "/set-name" => {
                match request.query {
                    None => Response::input("Set name".to_string()),
                    Some(name) => {
                        match storage.update_name(user, name) {
                            Ok(_user) => Response::redirect_temporary("/".to_string()),
                            Err(_) => Response::temporary_failure("Failed to update user".into())
                        }
                    }
                }
            }
            "/fight" => {
                let health = user.health - 1;
                if health < 0 { return Response::redirect_temporary("/".to_string()); }
                match storage.update_health(user, health) {
                    Ok(_user) => Response::redirect_temporary("/".to_string()),
                    Err(_) => Response::temporary_failure("Failed to update user".into())
                }
            }
            "/rest" => {
                let health = user.health + 1;
                if health > user.max_health { return Response::redirect_temporary("/".to_string()); }
                match storage.update_health(user, health) {
                    Ok(_user) => Response::redirect_temporary("/".to_string()),
                    Err(_) => Response::temporary_failure("Failed to update user".into())
                }
            }
            _ => Response::not_found("".to_string())
        }
    }
}