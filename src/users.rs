use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
};
use rusqlite::{Connection, OpenFlags, OptionalExtension};

#[derive(Debug)]
pub(crate) struct User {
    pub(crate) username: String,
    pwhash: Option<String>,
    contact: String,
}

fn open_users() -> Connection {
    Connection::open_with_flags(crate::config::USERDB, OpenFlags::SQLITE_OPEN_READ_WRITE).unwrap()
}

pub(crate) fn get_user(username: &str) -> Option<User> {
    open_users()
        .query_row(
            "SELECT username, pwhash, contact FROM users WHERE username COLLATE NOCASE = ?",
            [&username],
            |r| {
                Ok(User {
                    username: r.get("username")?,
                    pwhash: r.get("pwhash")?,
                    contact: r.get("contact")?,
                })
            },
        )
        .optional()
        .unwrap()
}

pub(crate) fn try_login(mut user: User, password: &str) -> Option<User> {
    Argon2::default()
        .verify_password(
            password.as_bytes(),
            &PasswordHash::new(&user.pwhash.take()?).unwrap(),
        )
        .ok()
        .and(Some(user))
}

pub(crate) fn change_password(username: &str, password: &str) {
    let pwhash = Argon2::default()
        .hash_password(password.as_bytes(), &SaltString::generate(&mut OsRng))
        .unwrap()
        .to_string();

    open_users()
        .execute(
            "UPDATE users SET pwhash = ? WHERE username = ?",
            [&pwhash, username],
        )
        .unwrap();
}

pub(crate) fn change_contact(username: &str, contact: &str) {
    open_users()
        .execute(
            "UPDATE users SET contact = ? WHERE username = ?",
            [contact, username],
        )
        .unwrap();
}

pub(crate) fn register(username: &str, password: &str, contact: &str) -> User {
    let pwhash = Argon2::default()
        .hash_password(password.as_bytes(), &SaltString::generate(&mut OsRng))
        .unwrap()
        .to_string();

    open_users()
        .execute(
            "INSERT INTO users (username, pwhash, contact) VALUES (?, ?, ?)",
            [username, &pwhash, contact],
        )
        .unwrap();

    get_user(username).unwrap()
}
