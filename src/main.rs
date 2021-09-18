mod ui;
mod users;

enum MenuAction<'a> {
    GoTo(&'a str),
    Return,
    Quit,
    Register,
    Login,
    ChangePassword,
    ChangeEmail,
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
                name: "change current email address",
                action: MenuAction::ChangeEmail,
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
        println!("\x1bc\n ## ascension.run - public NetHack server");
        match &user_cur {
            Some(user) => println!(" ## currently logged in as: {}", user.username),
            None => println!(" ## not logged in"),
        }
        print!("\n {}:\n", menu_cur.title);
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

                let new_email = ui::trimmed_input("new email > ");
                let confirm_email = ui::trimmed_input("confirm new email > ");
                if new_email != confirm_email {
                    ui::flash_error("the emails don't match");
                    continue;
                }
                if new_email.is_empty() {
                    ui::flash_error(
                        "you won't be able to ask for a password reset with no email on record!",
                    );
                } else if !new_email.contains('@') {
                    ui::flash_error(
                        "the new email address doesn't contain a @, please make sure it's correct!",
                    )
                }

                user_cur = Some(users::register(&new_username, &new_password, &new_email));
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

            MenuAction::ChangeEmail => {
                assert!(user_cur.is_some());

                let new_email = ui::trimmed_input("new email > ");
                let confirm_email = ui::trimmed_input("confirm new email > ");
                if new_email != confirm_email {
                    ui::flash_error("the emails don't match");
                    continue;
                }
                if new_email.is_empty() {
                    ui::flash_error("deleting email address on record - you won't be able to ask for a password reset!");
                } else if !new_email.contains('@') {
                    ui::flash_error(
                        "the new email address doesn't contain a @, please make sure it's correct!",
                    )
                }
                users::change_email(&user_cur.as_ref().unwrap().username, &new_email);
            }

            MenuAction::Play(_) => todo!(),
            MenuAction::EditRc(_) => todo!(),
            MenuAction::Watch => todo!(),
        }
    }

    print!("\x1bc");
}
