mod ui;
mod users;

enum MenuAction<'a> {
    GoTo(&'a str),
    Return,
    Quit,
    Register,
    Login,
    ChangePassword,
    ChangeContact,
    Play(&'a str),
    EditRc(&'a str),
    Watch,
}

struct MenuEntry<'a> {
    key: char,
    name: &'a str,
    action: MenuAction<'a>,
}

struct Menu<'a> {
    id: &'a str,
    title: &'a str,
    entries: &'a [MenuEntry<'a>],
}

const MENUS: &[Menu] = &[
    Menu {
        id: "mainmenu_anon",
        title: "Main menu",
        entries: &[
            MenuEntry {
                key: 'l',
                name: "login",
                action: MenuAction::Login,
            },
            MenuEntry {
                key: 'r',
                name: "register",
                action: MenuAction::Register,
            },
            MenuEntry {
                key: 'w',
                name: "watch games in progress",
                action: MenuAction::Watch,
            },
            MenuEntry {
                key: 'q',
                name: "quit",
                action: MenuAction::Quit,
            },
        ],
    },
    Menu {
        id: "mainmenu_user",
        title: "Main menu",
        entries: &[
            MenuEntry {
                key: 'c',
                name: "change current password",
                action: MenuAction::ChangePassword,
            },
            MenuEntry {
                key: 'e',
                name: "change current contact information",
                action: MenuAction::ChangeContact,
            },
            MenuEntry {
                key: 'n',
                name: "NetHack 3.4.3",
                action: MenuAction::GoTo("nethack"),
            },
            MenuEntry {
                key: 'w',
                name: "watch games in progress",
                action: MenuAction::Watch,
            },
            MenuEntry {
                key: 'q',
                name: "quit",
                action: MenuAction::Quit,
            },
        ],
    },
    Menu {
        id: "nethack",
        title: "NetHack 3.4.3",
        entries: &[
            MenuEntry {
                key: 'p',
                name: "play",
                action: MenuAction::Play("nethack"),
            },
            MenuEntry {
                key: 'e',
                name: "edit nethackrc",
                action: MenuAction::EditRc("nethack/nethackrc"),
            },
            MenuEntry {
                key: 'q',
                name: "back",
                action: MenuAction::Return,
            },
        ],
    },
];

fn main() {
    let menus = MENUS;
    let mut menu_hist = Vec::new();
    let mut menu_cur = &menus[0];
    let mut user_cur: Option<users::User> = None;

    loop {
        println!("\x1bc\n ## ascension.run - public NetHack server\n ##");
        match &user_cur {
            Some(user) => println!(" ## logged in as: {}", user.username),
            None => println!(" ## not logged in"),
        }
        println!("\n {}:", menu_cur.title);
        for entry in menu_cur.entries {
            println!("  {}) {}", entry.key, entry.name);
        }

        let choice = ui::char_input("\n > ");
        let choice = menu_cur.entries.iter().find(|&entry| choice == entry.key);
        let choice = match choice {
            Some(choice) => &choice.action,
            None => continue,
        };
        println!();

        match *choice {
            MenuAction::GoTo(id) => {
                menu_hist.push(menu_cur);
                menu_cur = menus.iter().find(|&menu| menu.id == id).unwrap();
            }
            MenuAction::Return => {
                menu_cur = menu_hist.pop().unwrap();
            }

            MenuAction::Quit => {
                break;
            }

            MenuAction::Register => {
                assert!(user_cur.is_none());

                let new_username = ui::trimmed_input("username > ");
                if new_username.is_empty() {
                    continue;
                }

                if !new_username.chars().all(|c| c.is_ascii_alphanumeric())
                    || new_username.len() > 15
                {
                    ui::flash_error(
                        "username must be ASCII alphanumeric and no longer than 15 characters",
                    );
                    continue;
                }

                if users::get_user(&new_username).is_some() {
                    ui::flash_error("user already exists");
                    continue;
                }

                let new_password = ui::pass_input("new password > ");
                let confirm_password = ui::pass_input("confirm new password > ");
                if new_password != confirm_password {
                    ui::flash_error("the passwords don't match");
                    continue;
                }

                let new_contact = ui::trimmed_input("contact information (email, IRC, discord) > ");
                if new_contact.is_empty() {
                    ui::flash_error(
                        "you won't be able to ask for a password reset with no contact information on record!",
                    );
                }

                user_cur = Some(users::register(&new_username, &new_password, &new_contact));
                menu_cur = &menus[1];
                menu_hist.clear();
            }

            MenuAction::Login => {
                assert!(user_cur.is_none());

                let tentative_username = ui::trimmed_input("username > ");
                if tentative_username.is_empty() {
                    continue;
                }
                let user = match users::get_user(&tentative_username) {
                    Some(user) => user,
                    None => {
                        ui::flash_error("no such user");
                        continue;
                    }
                };

                let tentative_password = ui::pass_input("password > ");
                user_cur = match users::try_login(user, &tentative_password) {
                    Some(user) => Some(user),
                    None => {
                        ui::flash_error("login error");
                        continue;
                    }
                };

                menu_cur = &menus[1];
                menu_hist.clear();
            }

            MenuAction::ChangePassword => {
                assert!(user_cur.is_some());

                let new_password = ui::pass_input("new password > ");
                let confirm_password = ui::pass_input("confirm new password > ");
                if new_password != confirm_password {
                    ui::flash_error("the passwords don't match");
                    continue;
                }
                users::change_password(&user_cur.as_ref().unwrap().username, &new_password);
            }

            MenuAction::ChangeContact => {
                assert!(user_cur.is_some());

                let new_contact = ui::trimmed_input("contact information (email, IRC, discord) > ");
                if new_contact.is_empty() {
                    ui::flash_error(
                        "deleting contact information - you won't be able to ask for a password reset!",
                    );
                }
                users::change_contact(&user_cur.as_ref().unwrap().username, &new_contact);
            }

            MenuAction::Play(_) => run_game(),
            MenuAction::EditRc(_) => run_editor(),
            MenuAction::Watch => todo!(),
        }
    }

    print!("\x1bc");
}

fn run_editor() {
    use std::os::unix::process::CommandExt;

    std::process::Command::new("nano")
        .arg0("rnano")
        .arg("nethackrc")
        .spawn()
        .unwrap()
        .wait()
        .unwrap();
}

fn run_game() {
    use std::io::{stdout, Read, Write};

    let mut rec = std::fs::File::create("test.ttyrec").unwrap();

    let mut child = std::process::Command::new("robotfindskitten")
        .stdout(std::process::Stdio::piped())
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

        stdout().write_all(&buf).unwrap();
        stdout().flush().unwrap();

        childout = childout_nb.into_blocking().unwrap();
    }

    child.wait().unwrap();
}

fn unix_duration() -> std::time::Duration {
    std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .unwrap()
}
