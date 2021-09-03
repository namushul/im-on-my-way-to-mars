use std::error;

use postgres::{Client, NoTls};

#[derive(Debug)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub max_health: i32,
    pub health: i32,
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
        const CONFIG: &str = "host=localhost user=postgres password=postgres";
        Ok(Storage { client: Client::connect(CONFIG, NoTls)? })
    }

    pub fn get_or_create_user(&mut self, fingerprint: &[u8]) -> Result<User, Error> {
        match self.get_user(fingerprint) {
            Ok(user) => Ok(user),
            Err(Error::NotFound) => Ok(self.create_user(fingerprint)?),
            Err(error) => Err(error)
        }
    }

    pub fn create_user(&mut self, fingerprint: &[u8]) -> Result<User, Error> {
        let name = "Alien".to_string();
        let max_health = 10;
        let health = 10;
        match self.client.query(
            "insert into users (fingerprint, name, max_health, health) values ($1, $2, $3, $4) RETURNING id",
            &[&fingerprint, &name, &max_health, &health],
        )?.first() {
            Some(row) => {
                let id = row.get(0);
                Ok(User { id, name, max_health, health })
            }
            None => Err(Error::MissingPrimaryKeyRow)
        }
    }

    pub fn get_user(&mut self, fingerprint: &[u8]) -> Result<User, Error> {
        match self.client.query("select id, name, max_health, health from users where fingerprint = $1", &[&fingerprint])?.first() {
            Some(row) => {
                let id = row.get(0);
                let name = row.get(1);
                let max_health = row.get(2);
                let health = row.get(3);
                Ok(User { id, name, max_health, health })
            }
            None => Err(Error::NotFound)
        }
    }

    pub fn update_name(&mut self, user: User, name: String) -> Result<User, Error> {
        self.client.execute(
            "update users set name = $1 where id = $2",
            &[&name, &user.id],
        )?;
        Ok(User { name, ..user })
    }

    pub fn update_health(&mut self, user: User, health: i32) -> Result<User, Error> {
        self.client.execute(
            "update users set health = $1 where id = $2",
            &[&health, &user.id],
        )?;
        Ok(User { health, ..user })
    }
}