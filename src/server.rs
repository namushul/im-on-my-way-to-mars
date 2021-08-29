use std::time::Duration;

use postgres::{Client, NoTls, Row};
use postgres_types::{FromSql, ToSql};
use ring::digest::Digest;
use url::Url;

use crate::banner::BANNER;
use crate::response::Response;

#[derive(Debug)]
pub struct Server {}

#[derive(Debug)]
pub struct Request {
    pub url: Url,
    pub query: Option<String>,
    pub user_fingerprint: Digest,
}

fn serve_frontpage(request: Request, user: User) -> Response {
    Response::success("text/gemini; lang=en".to_string(), format!("{}\r\n### 🐉 Hello, {}!\r\nHP: {}\r\n=> set-name Set name\r\n### Actions\r\n=> fight ⚔ Fight\r\n=> rest 🏥 Rest", BANNER, user.name, user.health))
}

#[derive(Debug)]
struct User {
    id: i32,
    name: String,
    max_health: i32,
    health: i32,
}

impl Server {
    pub fn handle_request(&self, request: Request) -> Response {
        // todo: db pooling
        let mut client = Client::connect("host=localhost user=postgres password=postgres", NoTls).unwrap();

        let fingerprint = request.user_fingerprint.as_ref();

        let user = match client.query("select id, name, max_health, health from users where fingerprint = $1", &[&fingerprint]).unwrap().first() {
            Some(row) => {
                let id = row.get(0);
                let name = row.get(1);
                let max_health = row.get(2);
                let health = row.get(3);
                User { id, name, max_health, health }
            }
            None => {
                let name = "Alien".to_string();
                let max_health = 10;
                let health = 10;
                match client.query(
                    "insert into users (fingerprint, name, max_health, health) values ($1, $2, $3, $4) RETURNING id",
                    &[&fingerprint, &name, &max_health, &health],
                ).unwrap().first() {
                    Some(row) => {
                        let id = row.get(0);

                        User { id, name, max_health, health }
                    }
                    None => {
                        panic!("Expected id after insert!");
                    }
                }
            }
        };
        eprintln!("User: {:?}", user);

        match request.url.path() {
            "/" | "" => {
                serve_frontpage(request, user)
            }
            "/set-name" => {
                match request.query {
                    None => Response::input("Set name".to_string()),
                    Some(name) => {
                        client.execute(
                            "update users set name = $1 where id = $2",
                            &[&name, &user.id],
                        ).unwrap();
                        let user = User { name, ..user };

                        Response::redirect_temporary("/".to_string())
                    }
                }
            }
            "/fight" => {
                let health = user.health - 1;
                client.execute(
                    "update users set health = $1 where id = $2",
                    &[&health, &user.id],
                ).unwrap();
                let user = User { health, ..user };

                Response::redirect_temporary("/".to_string())
            }
            "/rest" => {
                let health = user.health + 1;
                client.execute(
                    "update users set health = $1 where id = $2",
                    &[&health, &user.id],
                ).unwrap();
                let user = User { health, ..user };

                Response::redirect_temporary("/".to_string())
            }
            _ => Response::not_found("".to_string())
        }
    }
}