use std::{
    fs::File,
    io::Read,
};

use crate::spoke::terminal::Terminal;

fn open_pty() -> std::io::Result<(File, File)> {
    use std::os::fd::{FromRawFd, IntoRawFd};

    let pty =
        rustix::pty::openpt(rustix::pty::OpenptFlags::RDWR | rustix::pty::OpenptFlags::NOCTTY)?;

    rustix::pty::grantpt(&pty)?;
    rustix::pty::unlockpt(&pty)?;

    let slave_name = rustix::pty::ptsname(&pty, Vec::new())?;
    let slave = rustix::fs::open(
        slave_name,
        rustix::fs::OFlags::RDWR | rustix::fs::OFlags::NOCTTY,
        rustix::fs::Mode::empty(),
    )?;

    let master = unsafe { File::from_raw_fd(pty.into_raw_fd()) };
    let slave = unsafe { File::from_raw_fd(slave.into_raw_fd()) };

    Ok((master, slave))
}

fn read_exact_bytes(master: &mut File, len: usize) -> Vec<u8> {
    let mut buf = vec![0; len];
    master.read_exact(&mut buf).unwrap();
    buf
}

#[test]
fn new_switches_terminal_to_raw_mode_and_reset_restores_it() {
    let (_master, slave) = open_pty().unwrap();
    let slave_check = slave.try_clone().unwrap();

    let before = rustix::termios::tcgetattr(&slave_check).unwrap();
    let mut terminal = Terminal::new(slave).unwrap();

    let raw = rustix::termios::tcgetattr(&slave_check).unwrap();
    assert!(!raw.local_modes.contains(rustix::termios::LocalModes::ICANON));
    assert!(!raw.local_modes.contains(rustix::termios::LocalModes::ECHO));
    assert!(!raw.local_modes.contains(rustix::termios::LocalModes::ISIG));
    assert_eq!(raw.special_codes[rustix::termios::SpecialCodeIndex::VMIN], 1);
    assert_eq!(raw.special_codes[rustix::termios::SpecialCodeIndex::VTIME], 0);

    terminal.reset().unwrap();

    let after = rustix::termios::tcgetattr(&slave_check).unwrap();
    assert_eq!(before.local_modes, after.local_modes);
}

#[test]
fn write_str_writes_insert_sequence_followed_by_the_input() {
    let (mut master, slave) = open_pty().unwrap();
    let terminal = Terminal::new(slave).unwrap();

    terminal.write_str("abc").unwrap();

    let bytes = read_exact_bytes(&mut master, 7);
    assert_eq!(bytes, b"\x1b[3@abc");
}

#[test]
fn write_uses_a_single_character_insert_sequence() {
    let (mut master, slave) = open_pty().unwrap();
    let terminal = Terminal::new(slave).unwrap();

    terminal.write('é').unwrap();

    let bytes = read_exact_bytes(&mut master, 6);
    assert_eq!(bytes, b"\x1b[1@\xC3\xA9");
}

#[test]
fn write_with_prompt_rewrites_the_entire_line() {
    let (mut master, slave) = open_pty().unwrap();
    let terminal = Terminal::new(slave).unwrap();

    terminal.write_with_prompt("> ", "hello".to_string()).unwrap();

    let bytes = read_exact_bytes(&mut master, 12);
    assert_eq!(bytes, b"\r\x1b[2K> hello");
}

#[test]
fn cursor_and_editing_commands_write_the_expected_escape_sequences() {
    let (mut master, slave) = open_pty().unwrap();
    let terminal = Terminal::new(slave).unwrap();

    terminal.cursor_left().unwrap();
    terminal.cursor_right().unwrap();
    terminal.backspace().unwrap();
    terminal.delete().unwrap();

    let bytes = read_exact_bytes(&mut master, 12);
    assert_eq!(bytes, b"\x1b[D\x1b[C\x08 \x08\x1b[P");
}

#[test]
fn new_line_writes_a_single_newline_byte() {
    let (mut master, slave) = open_pty().unwrap();
    let terminal = Terminal::new(slave).unwrap();

    terminal.new_line().unwrap();

    let bytes = read_exact_bytes(&mut master, 1);
    assert_eq!(bytes, b"\r");
}
