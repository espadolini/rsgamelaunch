use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
};
use rusqlite::{Connection, OpenFlags, OptionalExtension};

#[derive(Debug)]
pub(crate) struct User {
    pub(crate) username: String,
    password: String,
    contact: String,
    nologin: bool,
}

fn open_users() -> Connection {
    Connection::open_with_flags("users.db", OpenFlags::SQLITE_OPEN_READ_WRITE).unwrap()
}

pub(crate) fn get_user(username: &str) -> Option<User> {
    open_users()
        .query_row(
            "SELECT username, password, contact, nologin FROM users WHERE username COLLATE NOCASE = ?",
            [&username],
            |r| {
                Ok(User {
                    username: r.get("username")?,
                    password: r.get("password")?,
                    contact: r.get("contact")?,
                    nologin: r.get("nologin")?,
                })
            },
        )
        .optional()
        .unwrap()
}

pub(crate) fn try_login(user: User, password: &str) -> Option<User> {
    if user.nologin {
        return None;
    }

    Argon2::default()
        .verify_password(
            password.as_bytes(),
            &PasswordHash::new(&user.password).unwrap(),
        )
        .ok()?;

    Some(user)
}

pub(crate) fn change_password(username: &str, password: &str) {
    let hashed_password = Argon2::default()
        .hash_password(password.as_bytes(), &SaltString::generate(&mut OsRng))
        .unwrap()
        .to_string();

    open_users()
        .execute(
            "UPDATE users SET password = ? WHERE username = ?",
            [hashed_password.as_str(), username],
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
    let hashed_password = Argon2::default()
        .hash_password(password.as_bytes(), &SaltString::generate(&mut OsRng))
        .unwrap()
        .to_string();

    open_users()
        .execute(
            "INSERT INTO users (username, password, contact, nologin) VALUES (?, ?, ?, 0)",
            [username, &hashed_password, contact],
        )
        .unwrap();

    get_user(username).unwrap()
}
