use std::path::{Path, PathBuf};

mod ui;
mod users;

enum OverwriteBehavior {
    OverwriteExisting,
    IgnoreExisting,
}

enum Action<'a> {
    GoTo(&'a str),
    Return,
    Quit,
    FlashMessage(&'a str),

    Register,
    Login,
    ChangePassword,
    ChangeContact,

    RunGame(&'a str),
    EditFile(&'a str),
    CopyFile(&'a str, &'a str, OverwriteBehavior),

    Watch,
}

struct Entry<'a> {
    key: char,
    name: &'a str,
    actions: &'a [Action<'a>],
}

struct Menu<'a> {
    id: &'a str,
    title: &'a str,
    entries: &'a [Entry<'a>],
}

const MENUS: &[Menu] = &[
    Menu {
        id: "mainmenu_anon",
        title: "Main menu",
        entries: &[
            Entry {
                key: 'l',
                name: "login",
                actions: &[Action::Login],
            },
            Entry {
                key: 'r',
                name: "register",
                actions: &[Action::Register],
            },
            Entry {
                key: 'w',
                name: "watch games in progress",
                actions: &[Action::Watch],
            },
            Entry {
                key: 'q',
                name: "quit",
                actions: &[Action::Quit],
            },
        ],
    },
    Menu {
        id: "mainmenu_user",
        title: "Main menu",
        entries: &[
            Entry {
                key: 'c',
                name: "change current password",
                actions: &[Action::ChangePassword],
            },
            Entry {
                key: 'e',
                name: "change current contact information",
                actions: &[Action::ChangeContact],
            },
            Entry {
                key: 'n',
                name: "NetHack 3.4.3",
                actions: &[Action::GoTo("nethack")],
            },
            Entry {
                key: 'w',
                name: "watch games in progress",
                actions: &[Action::Watch],
            },
            Entry {
                key: 'q',
                name: "quit",
                actions: &[Action::Quit],
            },
        ],
    },
    Menu {
        id: "nethack",
        title: "NetHack 3.4.3",
        entries: &[
            Entry {
                key: 'p',
                name: "play",
                actions: &[
                    Action::CopyFile(
                        "rgldir/nethackrc",
                        "nethack/nethackrc",
                        OverwriteBehavior::IgnoreExisting,
                    ),
                    Action::RunGame("nethack"),
                ],
            },
            Entry {
                key: 'e',
                name: "edit nethackrc",
                actions: &[
                    Action::CopyFile(
                        "rgldir/nethackrc",
                        "nethack/nethackrc",
                        OverwriteBehavior::IgnoreExisting,
                    ),
                    Action::EditFile("nethack/nethackrc"),
                ],
            },
            Entry {
                key: 'r',
                name: "reset nethackrc",
                actions: &[Action::GoTo("nethack_reset")],
            },
            Entry {
                key: 'q',
                name: "back",
                actions: &[Action::Return],
            },
        ],
    },
    Menu {
        id: "nethack_reset",
        title: "NetHack 3.4.3 rc file reset",
        entries: &[
            Entry {
                key: 'R',
                name: "confirm reset",
                actions: &[
                    Action::CopyFile(
                        "rgldir/nethackrc",
                        "nethack/nethackrc",
                        OverwriteBehavior::OverwriteExisting,
                    ),
                    Action::FlashMessage("the nethackrc file was reset to default"),
                    Action::Return,
                ],
            },
            Entry {
                key: 'q',
                name: "back",
                actions: &[Action::Return],
            },
        ],
    },
];

fn main() {
    std::panic::set_hook(Box::new(|p| eprintln!("{}", p)));

    let menus = MENUS;
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
        for entry in menu_cur.entries {
            println!("  {}) {}", entry.key, entry.name);
        }

        let choice = ui::char_input("\n > ");
        let entry = menu_cur.entries.iter().find(|&entry| choice == entry.key);
        let actions = match entry {
            Some(entry) => {
                println!("{}", choice);
                entry.actions
            }
            None => {
                println!("\x07");
                continue;
            }
        };

        for action in actions {
            match *action {
                Action::GoTo(id) => {
                    menu_hist.push(menu_cur);
                    menu_cur = menus.iter().find(|&menu| menu.id == id).unwrap();
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

                Action::RunGame(_) => run_game(),
                Action::EditFile(path) => {
                    let username = &user_cur.as_ref().unwrap().username;
                    let mut pathbuf = userdir(username);
                    pathbuf.push(path);

                    std::fs::create_dir_all(pathbuf.parent().unwrap()).unwrap();
                    run_editor(&pathbuf);
                }

                Action::CopyFile(src, dst, ref overwrite) => {
                    let username = &user_cur.as_ref().unwrap().username;
                    let mut pathbuf = userdir(username);
                    pathbuf.push(dst);

                    if matches!(overwrite, OverwriteBehavior::OverwriteExisting)
                        || !pathbuf.exists()
                    {
                        std::fs::create_dir_all(pathbuf.parent().unwrap()).unwrap();
                        std::fs::copy(src, pathbuf).unwrap();
                    }
                }

                Action::Watch => todo!(),
            }
        }
    }
}

fn userdir(username: &str) -> PathBuf {
    let pathbuf = Path::new("rgldir/userdata").join(username);
    std::fs::create_dir_all(&pathbuf).unwrap();
    pathbuf
}

fn run_editor(path: &Path) {
    use std::os::unix::process::CommandExt;

    std::process::Command::new("nano")
        .arg0("rnano")
        .arg(path)
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
