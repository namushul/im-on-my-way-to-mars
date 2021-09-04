//! Communication with the database only happens through this module.

use std::{env, error};

use postgres::{Client, NoTls};

pub mod locations {
    pub const BASTOW: i32 = 0;
    pub const BASTOW_WOODLANDS: i32 = 1;
}

#[derive(Debug)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub max_health: i32,
    pub health: i32,
    pub location_id: i32,
}

pub struct Storage {
    client: Client,
}

#[derive(Debug)]
pub enum Error {
    Db(Box<dyn error::Error + Sync + Send>),
    MissingPrimaryKeyRow,
    NotFound,
}

impl From<postgres::Error> for Error {
    fn from(e: postgres::Error) -> Self { Error::Db(Box::new(e)) }
}

impl Storage {
    // TODO: db pooling
    pub fn new() -> Result<Storage, Error> {
        let host = env::var("POSTGRES_HOST").unwrap_or("localhost".to_string());
        let user = env::var("POSTGRES_USER").unwrap_or("postgres".to_string());
        let password = env::var("POSTGRES_PASSWORD").unwrap_or("postgres".to_string());
        Ok(Storage { client: Client::connect(format!("host={} user={} password={}", host, user, password).as_str(), NoTls)? })
    }

    pub fn create_user(&mut self, fingerprint: &[u8], name: String) -> Result<User, Error> {
        let max_health = 10;
        let health = 10;
        let location_id = locations::BASTOW;
        match self.client.query(
            "insert into users (fingerprint, name, max_health, health, location_id) values ($1, $2, $3, $4, $5) RETURNING id",
            &[&fingerprint, &name, &max_health, &health, &location_id],
        )?.first() {
            Some(row) => {
                let id = row.get(0);
                Ok(User { id, name, max_health, health, location_id })
            }
            None => Err(Error::MissingPrimaryKeyRow)
        }
    }

    pub fn get_user(&mut self, fingerprint: &[u8]) -> Result<User, Error> {
        match self.client.query("select id, name, max_health, health, location_id from users where fingerprint = $1", &[&fingerprint])?.first() {
            Some(row) => {
                let id = row.get(0);
                let name = row.get(1);
                let max_health = row.get(2);
                let health = row.get(3);
                let location_id = row.get(4);
                Ok(User { id, name, max_health, health, location_id })
            }
            None => Err(Error::NotFound)
        }
    }

    pub fn update_health(&mut self, user: User, health: i32) -> Result<User, Error> {
        self.client.execute(
            "update users set health = $1 where id = $2",
            &[&health, &user.id],
        )?;
        Ok(User { health, ..user })
    }

    pub fn update_location_id(&mut self, user: User, location_id: i32) -> Result<User, Error> {
        self.client.execute(
            "update users set location_id = $1 where id = $2",
            &[&location_id, &user.id],
        )?;
        Ok(User { location_id, ..user })
    }
}