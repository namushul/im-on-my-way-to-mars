use url::Url;

use crate::response::Response;
use crate::storage::{locations, Storage, User};
use crate::storage;

#[derive(Debug)]
pub struct Server {}

#[derive(Debug)]
pub struct Request {
    pub url: Url,
    pub query: Option<String>,
    pub peer_fingerprint: Option<[u8; 32]>,
}

const BANNER: &str = include_str!("banner.txt");

fn serve_landing() -> Response {
    Response::success(
        "text/gemini; lang=en".to_string(),
        format!("{banner}\r\nYou have reached the enchanted land of Namushul.\r\n\r\nAre you ready to begin your adventure?\r\n=> join Enter", banner = BANNER),
    )
}

// TODO: => set-name 📝 Set name\r\n

fn bastow(user: User) -> Response {
    Response::success(
        "text/gemini; lang=en".to_string(),
        format!("\
        ### {name}\r\nHP: {health}/{max_health}\r\n\
        ### Bastow \r\nYou are in the small port town of Bastow. The town has an inn. A small gravel path leads out of town and into the forest.\r\n\
        ### Travel\r\n=> travel/{bastow_woodlands} 🌳 Follow the path into the forest.\r\n\
        ### Actions\r\n=> rest 🛏 Rest at the inn.", name = user.name, health = user.health, max_health = user.max_health, bastow_woodlands = locations::BASTOW_WOODLANDS),
    )
}

fn bastow_woodlands(user: User) -> Response {
    Response::success(
        "text/gemini; lang=en".to_string(),
        format!("\
        ### {name}\r\nHP: {health}/{max_health}\r\n\
        ### Bastow Woodlands \r\nYou are in Bastow Woodland. You see nothing of interest.\r\n\
        ### Travel\r\n=> travel/{bastow} 👣 Go back to Bastow.\r\n\
        ### Actions\r\n=> fight 👊 Fight slimes.", name = user.name, health = user.health, max_health = user.max_health, bastow = locations::BASTOW),
    )
}

fn status_page(user: User) -> Response {
    match user.location_id {
        locations::BASTOW_WOODLANDS => bastow_woodlands(user),
        locations::BASTOW | _ => bastow(user),
    }
}


impl Server {
    pub fn handle_request(&self, request: Request) -> Response {
        eprintln!("Request: {}", request.url);
        eprintln!("Request-path: {}", request.url.path());
        eprintln!("Request-query: {:?}", request.url.query_pairs().map(|(k, v)| { format!("{}: {}", k, v) }).collect::<Vec<String>>());
        eprintln!("Request-query: {:?}", request.query);

        let fingerprint = &match request.peer_fingerprint {
            Some(f) => f,
            None => {
                match request.url.path() {
                    "/" | "" => {
                        return serve_landing();
                    }
                    _ => return Response::client_certificate_required("Hello brave traveler. To venture further into this land you must present a certificate.".to_string())
                }
            }
        };

        let mut storage = match Storage::new() {
            Ok(storage) => storage,
            Err(_) => return Response::temporary_failure("Failed to connect to database".into()),
        };

        let user = match storage.get_user(fingerprint) {
            Ok(user) => Some(user),
            Err(storage::Error::NotFound) => None,
            Err(_) => return Response::temporary_failure("Failed to get/create user".into()),
        };
        eprintln!("User: {:?}", user);
        let user = match user {
            Some(user) => user,
            None => {
                match request.query {
                    None => return Response::input("Choose a name for your character".to_string()),
                    Some(name) => {
                        match storage.create_user(fingerprint, name) {
                            Ok(user) => user,
                            Err(_) => return Response::temporary_failure("Failed to create user".into())
                        }
                    }
                }
            }
        };

        let path_segments = match request.url.path_segments() {
            None => vec![],
            Some(segments) => segments.collect::<Vec<_>>()
        };
        eprintln!("{:?}", path_segments);
        match path_segments[..] {
            [""] | [] => {
                status_page(user)
            }
            ["join"] => {
                Response::redirect_temporary("/".to_string())
            }
            ["fight"] => {
                let health = user.health - 1;
                if health < 0 { return Response::redirect_temporary("/".to_string()); }
                match storage.update_health(user, health) {
                    Ok(_user) => Response::redirect_temporary("/".to_string()),
                    Err(_) => Response::temporary_failure("Failed to update user".into())
                }
            }
            ["rest"] => {
                let health = user.health + 1;
                if health > user.max_health { return Response::redirect_temporary("/".to_string()); }
                match storage.update_health(user, health) {
                    Ok(_user) => Response::redirect_temporary("/".to_string()),
                    Err(_) => Response::temporary_failure("Failed to update user".into())
                }
            }
            ["travel", destination_id] => {
                let destination_id = match destination_id.parse::<i32>() {
                    Ok(destination_id) => destination_id,
                    Err(_) => return Response::bad_request("Destination must be an integer".to_string())
                };
                match user.location_id {
                    locations::BASTOW => {
                        if destination_id != locations::BASTOW_WOODLANDS {
                            return Response::bad_request("Invalid destination".to_string());
                        }
                    }
                    locations::BASTOW_WOODLANDS => {
                        if destination_id != locations::BASTOW {
                            return Response::bad_request("Invalid destination".to_string());
                        }
                    }
                    _ => return Response::temporary_failure("Invalid user location".to_string())
                }
                match storage.update_location_id(user, destination_id) {
                    Ok(_user) => Response::redirect_temporary("/".to_string()),
                    Err(_) => Response::temporary_failure("Failed to update user".into())
                }
            }
            _ => Response::not_found("".to_string())
        }
    }
}