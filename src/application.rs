//! This handles the business logic, decoupled from the actual transport layer.

use std::env;
use std::thread::sleep;
use std::time::{Duration, Instant};

use url::Url;

use crate::duration::Humanize;
use crate::response::{Language, MediaType, Response};
use crate::storage::{locations, Storage, User};
use crate::storage;

#[derive(Debug)]
pub struct Application {
    start_time: Instant,
}

impl Application {
    pub fn new(start_time: Instant) -> Self {
        Self { start_time }
    }
}

#[derive(Debug)]
pub struct Request {
    pub url: Url,
    pub query: Option<String>,
    pub peer_fingerprint: Option<[u8; 32]>,
}

const BANNER: &str = include_str!("banner.txt");

fn serve_landing() -> Response {
    Response::success(
        MediaType::gemini(Some(Language::english())),
        format!("{banner}\r\nYou have reached the enchanted land of Namushul.\r\n\r\nAre you ready to begin your adventure?\r\n=> /adventure Enter\r\n=> /about About", banner = BANNER),
    )
}

// TODO: => set-name 📝 Set name\r\n

fn bastow(user: User) -> Response {
    Response::success(
        MediaType::gemini(Some(Language::english())),
        format!("\
        ### {name}\r\nHP: {health}/{max_health}\r\n\
        ### Bastow \r\nYou are in the small port town of Bastow. The town has an inn. A small gravel path leads out of town and into the forest.\r\n\
        ### Travel\r\n=> /adventure/bastow-woodlands 🌳 Follow the path into the forest.\r\n\
        ### Actions\r\n=> /adventure/rest 🛏 Rest at the inn.", name = user.name, health = user.health, max_health = user.max_health),
    )
}

fn bastow_woodlands(user: User) -> Response {
    Response::success(
        MediaType::gemini(Some(Language::english())),
        format!("\
        ### {name}\r\nHP: {health}/{max_health}\r\n\
        ### Bastow Woodlands \r\nYou are in Bastow Woodland. You see nothing of interest.\r\n\
        ### Travel\r\n=> /adventure/bastow 👣 Go back to Bastow.\r\n\
        ### Actions\r\n=> /adventure/fight 👊 Fight slimes.", name = user.name, health = user.health, max_health = user.max_health),
    )
}

fn status_page(user: User) -> Response {
    match user.location_id {
        locations::BASTOW_WOODLANDS => bastow_woodlands(user),
        locations::BASTOW | _ => bastow(user),
    }
}


impl Application {
    pub fn handle_request(&self, request: Request) -> Response {
        eprintln!("Request: {}", request.url);
        eprintln!("Request-path: {}", request.url.path());
        eprintln!("Request-query: {:?}", request.url.query_pairs().map(|(k, v)| { format!("{}: {}", k, v) }).collect::<Vec<String>>());
        eprintln!("Request-query: {:?}", request.query);

        let simulate_latency = env::var("SIMULATE_LATENCY").unwrap_or("false".to_owned());
        if simulate_latency == "true" {
            sleep(Duration::from_secs(1));
        }

        let path_segments = match request.url.path_segments() {
            None => vec![],
            Some(segments) => segments.collect::<Vec<_>>()
        };
        eprintln!("{:?}", path_segments);

        let mut storage = match Storage::new() {
            Ok(storage) => storage,
            Err(_) => return Response::temporary_failure("Failed to connect to database".into()),
        };

        match path_segments[..] {
            [""] | [] => {
                return serve_landing();
            }
            ["about"] => {
                let user_count = match storage.count_users() {
                    Ok(count) => count,
                    Err(_) => return Response::temporary_failure("Failed to count users".into()),
                };
                let fields = [
                    format!("👥 Users: {}", user_count),
                    // format!("🕐 Activity: {} ago", "1337 seconds"),
                    format!("🕗 Uptime: {}", self.start_time.elapsed().humanize()),
                ];
                return Response::success(
                    MediaType::gemini(Some(Language::english())),
                    format!("### About\r\n{}\r\n", fields.join(" · ")),
                );
            }
            _ => {}
        }

        let fingerprint = &match request.peer_fingerprint {
            Some(f) => f,
            None => {
                return Response::client_certificate_required("Hello brave traveler. To venture further into this land you must present a certificate.".to_owned());
            }
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
                    None => return Response::input("Choose a name for your character".to_owned()),
                    Some(name) => {
                        match storage.create_user(fingerprint, name) {
                            Ok(user) => user,
                            Err(_) => return Response::temporary_failure("Failed to create user".into())
                        }
                    }
                }
            }
        };

        match path_segments[..] {
            ["adventure"] => {
                status_page(user)
            }
            ["adventure", "fight"] => {
                let health = user.health - 1;
                if health < 0 { return Response::redirect_temporary("/adventure".to_owned()); }
                match storage.update_health(user, health) {
                    Ok(_user) => Response::redirect_temporary("/adventure".to_owned()),
                    Err(_) => Response::temporary_failure("Failed to update user".into())
                }
            }
            ["adventure", "rest"] => {
                let health = user.max_health;
                match storage.update_health(user, health) {
                    Ok(_user) => Response::redirect_temporary("/adventure".to_owned()),
                    Err(_) => Response::temporary_failure("Failed to update user".into())
                }
            }
            ["adventure", "bastow"] => {
                if user.location_id == locations::BASTOW {
                    return status_page(user);
                }
                if user.location_id != locations::BASTOW_WOODLANDS {
                    return Response::bad_request("Invalid destination".to_owned());
                }
                match storage.update_location_id(user, locations::BASTOW) {
                    Ok(user) => status_page(user),
                    Err(_) => Response::temporary_failure("Failed to update user".into())
                }
            }
            ["adventure", "bastow-woodlands"] => {
                if user.location_id == locations::BASTOW_WOODLANDS {
                    return status_page(user);
                }
                match storage.update_location_id(user, locations::BASTOW_WOODLANDS) {
                    Ok(user) => status_page(user),
                    Err(_) => Response::temporary_failure("Failed to update user".into())
                }
            }
            _ => Response::not_found("".to_owned())
        }
    }
}