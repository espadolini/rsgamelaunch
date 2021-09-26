use serde::Deserialize;
use std::{
    borrow::Cow,
    fs,
    io::{self, Read, Write},
    path::{Path, PathBuf},
    process,
    time::{Duration, SystemTime},
};

mod ui;
mod users;

const USERDIR_ROOT: &str = "rgldir/userdata";
const USERDB: &str = "rgldir/users.db";
const CONFIG_PATH: &str = "rgldir/config.ron";

#[derive(Deserialize)]
enum OverwriteBehavior {
    OverwriteExisting,
    IgnoreExisting,
}

#[derive(Deserialize)]
enum PathSpec {
    UserDir(String),
    Normal(String),
}

fn userdir(username: &str) -> PathBuf {
    let pathbuf = Path::new(USERDIR_ROOT).join(username);
    fs::create_dir_all(&pathbuf).unwrap();
    pathbuf
}

impl PathSpec {
    fn resolve(&self, username: &str) -> Cow<Path> {
        match self {
            Self::UserDir(p) => {
                let mut pb = userdir(username);
                pb.push(p);
                pb.into()
            }
            Self::Normal(p) => Path::new(p).into(),
        }
    }
}

#[derive(Deserialize)]
enum Action {
    GoTo(String),
    Return,
    Quit,
    FlashMessage(String),

    Register,
    Login,
    ChangePassword,
    ChangeContact,

    RunGame(String),
    EditFile(PathSpec),
    CopyFile(PathSpec, PathSpec, OverwriteBehavior),

    Watch,
}

#[derive(Deserialize)]
struct Entry {
    key: char,
    name: String,
    actions: Vec<Action>,
}

#[derive(Deserialize)]
struct Menu {
    id: String,
    title: String,
    entries: Vec<Entry>,
}

#[derive(Deserialize)]
struct Game {
    id: String,
}

#[derive(Deserialize)]
struct Config {
    menus: Vec<Menu>,
    games: Vec<Game>,
}

fn main() {
    std::panic::set_hook(Box::new(|p| eprintln!("{}", p)));

    let Config { menus, games } = ron::from_str(&fs::read_to_string(CONFIG_PATH).unwrap()).unwrap();

    let mut menu_hist = Vec::new();
    let mut menu_cur = &menus[0];
    let mut user_cur: Option<users::User> = None;

    'mainloop: loop {
        println!("\x1b[2J\x1b[H\x1bm\n ## ascension.run - public NetHack server\n ##");
        match &user_cur {
            Some(user) => println!(" ## logged in as: {}", user.username),
            None => println!(" ## not logged in"),
        }
        println!("\n {}:", menu_cur.title);
        for entry in &menu_cur.entries {
            println!("  {}) {}", entry.key, entry.name);
        }

        let choice = ui::char_input("\n > ");
        let entry = &menu_cur.entries.iter().find(|entry| choice == entry.key);
        let actions = match entry {
            Some(entry) => {
                println!("{}", choice);
                &entry.actions
            }
            None => {
                println!("\x07");
                continue;
            }
        };

        for action in actions {
            match action {
                Action::GoTo(id) => {
                    menu_hist.push(menu_cur);
                    menu_cur = menus.iter().find(|&menu| menu.id == *id).unwrap();
                }
                Action::Return => menu_cur = menu_hist.pop().unwrap(),

                Action::Quit => break 'mainloop,
                Action::FlashMessage(msg) => ui::flash_message(msg),

                Action::Register => {
                    assert!(user_cur.is_none());

                    let new_username = ui::trimmed_input("new username: ");
                    if new_username.is_empty() {
                        continue;
                    }

                    if !new_username.chars().all(|c| c.is_ascii_alphanumeric())
                        || new_username.len() > 15
                    {
                        ui::flash_message(
                            "username must be ASCII alphanumeric and no longer than 15 characters",
                        );
                        continue;
                    }

                    if users::get_user(&new_username).is_some() {
                        ui::flash_message("user already exists");
                        continue;
                    }

                    let new_password = ui::pass_input("new password: ");
                    let confirm_password = ui::pass_input("confirm new password: ");
                    if new_password != confirm_password {
                        ui::flash_message("the passwords don't match");
                        continue;
                    }

                    let new_contact =
                        ui::trimmed_input("contact information (email, IRC, discord): ");

                    user_cur = Some(users::register(&new_username, &new_password, &new_contact));

                    userdir(&user_cur.as_ref().unwrap().username);
                    menu_cur = &menus[1];
                    menu_hist.clear();
                }

                Action::Login => {
                    assert!(user_cur.is_none());

                    let tentative_username = ui::trimmed_input("username: ");
                    if tentative_username.is_empty() {
                        continue;
                    }
                    let user = match users::get_user(&tentative_username) {
                        Some(user) => user,
                        None => {
                            ui::flash_message("no such user");
                            continue;
                        }
                    };

                    let tentative_password = ui::pass_input("password: ");
                    user_cur = users::try_login(user, &tentative_password);
                    if user_cur.is_none() {
                        ui::flash_message("login error");
                        continue;
                    }

                    userdir(&user_cur.as_ref().unwrap().username);
                    menu_cur = &menus[1];
                    menu_hist.clear();
                }

                Action::ChangePassword => {
                    assert!(user_cur.is_some());

                    let new_password = ui::pass_input("new password: ");
                    let confirm_password = ui::pass_input("confirm new password: ");
                    if new_password != confirm_password {
                        ui::flash_message("the passwords don't match");
                        continue;
                    }
                    users::change_password(&user_cur.as_ref().unwrap().username, &new_password);
                }

                Action::ChangeContact => {
                    assert!(user_cur.is_some());

                    let new_contact =
                        ui::trimmed_input("contact information (email, IRC, discord): ");
                    if new_contact.is_empty() {
                        ui::flash_message("deleting contact information");
                    }
                    users::change_contact(&user_cur.as_ref().unwrap().username, &new_contact);
                }

                Action::RunGame(id) => run_game(
                    user_cur.as_ref().unwrap(),
                    games.iter().find(|g| &g.id == id).unwrap(),
                ),
                Action::EditFile(path) => {
                    let username = &user_cur.as_ref().unwrap().username;
                    let path = path.resolve(username);

                    fs::create_dir_all(path.parent().unwrap()).unwrap();
                    run_editor(&path);
                }

                Action::CopyFile(src, dst, overwrite) => {
                    let username = &user_cur.as_ref().unwrap().username;
                    let src = src.resolve(username);
                    let dst = dst.resolve(username);

                    if matches!(overwrite, OverwriteBehavior::OverwriteExisting) || !dst.exists() {
                        fs::create_dir_all(dst.parent().unwrap()).unwrap();
                        fs::copy(src, dst).unwrap();
                    }
                }

                Action::Watch => todo!(),
            }
        }
    }
}

fn run_editor(path: &Path) {
    process::Command::new("nano")
        .arg("--restricted")
        .arg(path)
        .spawn()
        .unwrap()
        .wait()
        .unwrap();
}

fn run_game(user: &users::User, game: &Game) {
    let _ = user;
    let _ = game;

    let mut rec = fs::File::create("test.ttyrec").unwrap();

    let mut child = process::Command::new("robotfindskitten")
        .stdout(process::Stdio::piped())
        .spawn()
        .unwrap();

    let mut childout = child.stdout.take().unwrap();

    let mut buf = Vec::with_capacity(16384);

    loop {
        buf.resize(1024, 0);
        let len = childout.read(&mut buf).unwrap();
        if len == 0 {
            break; // EOF
        }
        buf.truncate(len);
        let mut childout_nb = nonblock::NonBlockingReader::from_fd(childout).unwrap();
        childout_nb.read_available(&mut buf).unwrap();

        assert!(buf.len() <= u32::MAX as usize);

        let now = unix_duration();
        rec.write_all(&(now.as_secs() as u32).to_le_bytes())
            .unwrap();
        rec.write_all(&(now.subsec_micros() as u32).to_le_bytes())
            .unwrap();
        rec.write_all(&(buf.len() as u32).to_le_bytes()).unwrap();
        rec.write_all(&buf).unwrap();
        rec.flush().unwrap();

        io::stdout().write_all(&buf).unwrap();
        io::stdout().flush().unwrap();

        childout = childout_nb.into_blocking().unwrap();
    }

    child.wait().unwrap();
}

fn unix_duration() -> Duration {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
}
