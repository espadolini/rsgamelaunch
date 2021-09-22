use crate::{Action::*, Entry, Menu, OverwriteBehavior::*, PathSpec::*};

pub(crate) const USERDIR_ROOT: &str = "rgldir/userdata";
pub(crate) const USERDB: &str = "rgldir/users.db";

pub(crate) const MENUS: &[Menu] = &[
    Menu {
        id: "mainmenu_anon",
        title: "Main menu",
        entries: &[
            Entry {
                key: 'l',
                name: "login",
                actions: &[Login],
            },
            Entry {
                key: 'r',
                name: "register",
                actions: &[Register],
            },
            Entry {
                key: 'w',
                name: "watch games in progress",
                actions: &[Watch],
            },
            Entry {
                key: 'q',
                name: "quit",
                actions: &[Quit],
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
                actions: &[ChangePassword],
            },
            Entry {
                key: 'e',
                name: "change current contact information",
                actions: &[ChangeContact],
            },
            Entry {
                key: 'n',
                name: "NetHack 3.4.3",
                actions: &[GoTo("nethack")],
            },
            Entry {
                key: 'w',
                name: "watch games in progress",
                actions: &[Watch],
            },
            Entry {
                key: 'q',
                name: "quit",
                actions: &[Quit],
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
                    CopyFile(
                        Normal("rgldir/nethackrc"),
                        UserDir("nethack/nethackrc"),
                        IgnoreExisting,
                    ),
                    RunGame("nethack"),
                ],
            },
            Entry {
                key: 'e',
                name: "edit nethackrc",
                actions: &[
                    CopyFile(
                        Normal("rgldir/nethackrc"),
                        UserDir("nethack/nethackrc"),
                        IgnoreExisting,
                    ),
                    EditFile(UserDir("nethack/nethackrc")),
                ],
            },
            Entry {
                key: 'r',
                name: "reset nethackrc",
                actions: &[GoTo("nethack_reset")],
            },
            Entry {
                key: 'q',
                name: "back",
                actions: &[Return],
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
                    CopyFile(
                        Normal("rgldir/nethackrc"),
                        UserDir("nethack/nethackrc"),
                        OverwriteExisting,
                    ),
                    FlashMessage("the nethackrc file was reset to default"),
                    Return,
                ],
            },
            Entry {
                key: 'q',
                name: "back",
                actions: &[Return],
            },
        ],
    },
];
