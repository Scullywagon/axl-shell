use std::{
    io::Write,
    thread,
    time::Duration,
};

use crate::spoke::{Line, Reader};

fn open_pty() -> std::io::Result<(std::fs::File, std::fs::File)> {
    use std::fs::File;
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

#[test]
fn read_returns_response_when_enter_is_pressed() {
    let (mut master, slave) = open_pty().unwrap();

    let handle = thread::spawn(move || {
        let mut reader = Reader::new(slave, None).unwrap();
        reader.read("> ").unwrap()
    });

    thread::sleep(Duration::from_millis(10));
    master.write_all(b"hello\r").unwrap();

    let line = handle.join().unwrap();

    assert_eq!(line, Line::Res("hello".into()));
}

#[test]
fn read_returns_empty_line_res_when_enter_is_pressed_on_a_blank_prompt() {
    let (mut master, slave) = open_pty().unwrap();

    let handle = thread::spawn(move || {
        let mut reader = Reader::new(slave, None).unwrap();
        reader.read("> ").unwrap()
    });

    thread::sleep(Duration::from_millis(10));
    master.write_all(b"\r").unwrap();

    let line = handle.join().unwrap();

    assert_eq!(line, Line::Res("".into()));
}

#[test]
fn read_returns_line_eof_when_ctrl_d_is_pressed_on_an_empty_buffer() {
    let (mut master, slave) = open_pty().unwrap();

    let handle = thread::spawn(move || {
        let mut reader = Reader::new(slave, None).unwrap();
        reader.read("> ").unwrap()
    });

    thread::sleep(Duration::from_millis(10));
    master.write_all(&[0x04]).unwrap();

    let line = handle.join().unwrap();

    assert_eq!(line, Line::EoF);
}

#[test]
fn read_ignores_ctrl_d_when_the_buffer_is_not_empty() {
    let (mut master, slave) = open_pty().unwrap();

    let handle = thread::spawn(move || {
        let mut reader = Reader::new(slave, None).unwrap();
        reader.read("> ").unwrap()
    });

    thread::sleep(Duration::from_millis(10));
    master.write_all(b"woop").unwrap();
    master.write_all(&[0x04]).unwrap();
    master.write_all(b"\r").unwrap();

    let line = handle.join().unwrap();

    assert_eq!(line, Line::Res("woop".into()));
}

#[test]
fn read_returns_line_interrupt_when_ctrl_c_is_pressed() {
    let (mut master, slave) = open_pty().unwrap();

    let handle = thread::spawn(move || {
        let mut reader = Reader::new(slave, None).unwrap();
        reader.read("> ").unwrap()
    });

    thread::sleep(Duration::from_millis(10));
    master.write_all(b"woop").unwrap();
    master.write_all(&[0x03]).unwrap();

    let line = handle.join().unwrap();

    assert_eq!(line, Line::Interrupt);
}

#[test]
fn read_handles_backspace_after_typed_text_and_removes_the_last_character() {
    let (mut master, slave) = open_pty().unwrap();

    let handle = thread::spawn(move || {
        let mut reader = Reader::new(slave, None).unwrap();
        reader.read("> ").unwrap()
    });

    thread::sleep(Duration::from_millis(10));
    master.write_all(b"ab").unwrap();
    master.write_all(&[0x7F]).unwrap();
    master.write_all(b"\r").unwrap();

    let line = handle.join().unwrap();

    assert_eq!(line, Line::Res("a".into()));
}

#[test]
fn read_ignores_backspace_when_the_buffer_is_empty() {
    let (mut master, slave) = open_pty().unwrap();

    let handle = thread::spawn(move || {
        let mut reader = Reader::new(slave, None).unwrap();
        reader.read("> ").unwrap()
    });

    thread::sleep(Duration::from_millis(10));
    master.write_all(&[0x7F]).unwrap();
    master.write_all(b"x\r").unwrap();

    let line = handle.join().unwrap();

    assert_eq!(line, Line::Res("x".into()));
}

#[test]
fn read_moves_the_cursor_left_on_left_arrow_input() {
    let (mut master, slave) = open_pty().unwrap();

    let handle = thread::spawn(move || {
        let mut reader = Reader::new(slave, None).unwrap();
        reader.read("> ").unwrap()
    });

    thread::sleep(Duration::from_millis(10));
    master.write_all(b"ab").unwrap();
    master.write_all(&[0x1B, b'[', b'D']).unwrap();
    master.write_all(b"c\r").unwrap();

    let line = handle.join().unwrap();

    assert_eq!(line, Line::Res("acb".into()));
}

#[test]
fn read_moves_the_cursor_right_on_right_arrow_input() {
    let (mut master, slave) = open_pty().unwrap();

    let handle = thread::spawn(move || {
        let mut reader = Reader::new(slave, None).unwrap();
        reader.read("> ").unwrap()
    });

    thread::sleep(Duration::from_millis(10));
    master.write_all(b"ab").unwrap();
    master.write_all(&[0x1B, b'[', b'D']).unwrap();
    master.write_all(&[0x1B, b'[', b'C']).unwrap();
    master.write_all(b"c\r").unwrap();

    let line = handle.join().unwrap();

    assert_eq!(line, Line::Res("abc".into()));
}

#[test]
fn read_inserts_printable_ascii_at_the_current_cursor_position() {
    let (mut master, slave) = open_pty().unwrap();

    let handle = thread::spawn(move || {
        let mut reader = Reader::new(slave, None).unwrap();
        reader.read("> ").unwrap()
    });

    thread::sleep(Duration::from_millis(10));
    master.write_all(b"ac").unwrap();
    master.write_all(&[0x1B, b'[', b'D']).unwrap();
    master.write_all(b"b\r").unwrap();

    let line = handle.join().unwrap();

    assert_eq!(line, Line::Res("abc".into()));
}

#[test]
fn read_handles_delete_after_an_escape_sequence_when_the_cursor_is_not_at_the_end() {
    let (mut master, slave) = open_pty().unwrap();

    let handle = thread::spawn(move || {
        let mut reader = Reader::new(slave, None).unwrap();
        reader.read("> ").unwrap()
    });

    thread::sleep(Duration::from_millis(10));
    master.write_all(b"ab").unwrap();
    master.write_all(&[0x1B, b'[', b'D']).unwrap();
    master.write_all(&[0x1B, b'[', b'3', b'~']).unwrap();
    master.write_all(b"\r").unwrap();

    let line = handle.join().unwrap();

    assert_eq!(line, Line::Res("a".into()));
}

#[test]
fn read_leaves_the_buffer_unchanged_when_delete_is_pressed_at_the_end_of_the_buffer() {
    let (mut master, slave) = open_pty().unwrap();

    let handle = thread::spawn(move || {
        let mut reader = Reader::new(slave, None).unwrap();
        reader.read("> ").unwrap()
    });

    thread::sleep(Duration::from_millis(10));
    master.write_all(b"ab").unwrap();
    master.write_all(&[0x1B, b'[', b'3', b'~']).unwrap();
    master.write_all(b"\r").unwrap();

    let line = handle.join().unwrap();

    assert_eq!(line, Line::Res("ab".into()));
}

#[test]
fn read_resets_terminal_state_after_a_completed_line() {
    let (mut master, slave) = open_pty().unwrap();
    let slave_for_check = slave.try_clone().unwrap();
    let initial = rustix::termios::tcgetattr(&slave).unwrap();

    let handle = thread::spawn(move || {
        let mut reader = Reader::new(slave, None).unwrap();
        reader.read("> ").unwrap()
    });

    thread::sleep(Duration::from_millis(10));
    master.write_all(b"hello\r").unwrap();

    let line = handle.join().unwrap();
    assert_eq!(line, Line::Res("hello".into()));

    let reset = rustix::termios::tcgetattr(&slave_for_check).unwrap();
    assert_eq!(reset.local_modes, initial.local_modes);
    assert_eq!(
        reset.special_codes[rustix::termios::SpecialCodeIndex::VMIN],
        initial.special_codes[rustix::termios::SpecialCodeIndex::VMIN]
    );
    assert_eq!(
        reset.special_codes[rustix::termios::SpecialCodeIndex::VTIME],
        initial.special_codes[rustix::termios::SpecialCodeIndex::VTIME]
    );
}

#[test]
fn reader_resets_terminal_state_on_drop_even_if_read_exits_early() {
    let (_master, slave) = open_pty().unwrap();
    let slave_for_check = slave.try_clone().unwrap();
    let initial = rustix::termios::tcgetattr(&slave).unwrap();

    {
        let reader = Reader::new(slave, None).unwrap();
        drop(reader);
    }

    let reset = rustix::termios::tcgetattr(&slave_for_check).unwrap();
    assert_eq!(reset.local_modes, initial.local_modes);
    assert_eq!(
        reset.special_codes[rustix::termios::SpecialCodeIndex::VMIN],
        initial.special_codes[rustix::termios::SpecialCodeIndex::VMIN]
    );
    assert_eq!(
        reset.special_codes[rustix::termios::SpecialCodeIndex::VTIME],
        initial.special_codes[rustix::termios::SpecialCodeIndex::VTIME]
    );
}
