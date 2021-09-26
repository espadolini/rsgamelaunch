use std::{
    io::{self, Write},
    os::unix::io::{AsRawFd, RawFd},
    thread,
    time::Duration,
};
use termion::{event::Key, input::TermRead, raw::IntoRawMode};
use trim_in_place::TrimInPlace;

pub(crate) fn trimmed_input(prompt: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().unwrap();

    let mut out = String::new();
    io::stdin().read_line(&mut out).unwrap();
    out.trim_in_place();
    out
}

struct EchoOff {
    fd: RawFd,
    ios: termios::Termios,
}

impl EchoOff {
    fn new(fd: RawFd) -> Self {
        let ios = termios::Termios::from_fd(fd).unwrap();
        let mut new_ios = ios;
        new_ios.c_lflag &= !termios::ECHO;
        new_ios.c_lflag |= termios::ECHONL;
        termios::tcsetattr(fd, termios::TCSAFLUSH, &new_ios).unwrap();
        Self { fd, ios }
    }
}

impl Drop for EchoOff {
    fn drop(&mut self) {
        termios::tcsetattr(self.fd, termios::TCSAFLUSH, &self.ios).unwrap();
    }
}

pub(crate) fn pass_input(prompt: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().unwrap();

    let _noecho = EchoOff::new(io::stdout().as_raw_fd());
    let mut out = String::new();
    io::stdin().read_line(&mut out).unwrap();
    out.truncate(out.len() - 1);
    out
}

pub(crate) fn char_input(prompt: &str) -> char {
    print!("{}", prompt);
    io::stdout().flush().unwrap();

    let _raw = io::stdout().into_raw_mode().unwrap();
    for key in io::stdin().keys().flatten() {
        if let Key::Char(c) = key {
            return c;
        }
    }

    panic!("eof")
}

pub(crate) fn flash_message(msg: &str) {
    println!("{}", msg);
    thread::sleep(Duration::from_secs(2));
}
