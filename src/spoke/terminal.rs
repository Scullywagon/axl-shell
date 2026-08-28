use std::{io::Error, os::fd::AsFd};

use rustix::termios::{self, Termios};

pub struct Terminal<T: AsFd> {
    term: T,
    reset: Termios,
}

impl<T: AsFd> Terminal<T> {
    pub fn new(term: T) -> Result<Self, Error> {
        unsafe {
            libc::signal(libc::SIGINT, libc::SIG_IGN);
        }
        let reset = termios::tcgetattr(&term)?;

        let mut raw = reset.clone();

        raw.local_modes.remove(
            termios::LocalModes::ICANON | termios::LocalModes::ECHO | termios::LocalModes::ISIG,
        );

        raw.special_codes[termios::SpecialCodeIndex::VMIN] = 1;
        raw.special_codes[termios::SpecialCodeIndex::VTIME] = 0;

        termios::tcsetattr(&term, termios::OptionalActions::Now, &raw)?;

        Ok(Self { term, reset })
    }

    pub fn reset(&mut self) -> Result<(), Error> {
        termios::tcsetattr(&self.term, termios::OptionalActions::Now, &self.reset)?;

        Ok(())
    }

    pub fn read_bytes<const N: usize>(&self) -> Result<[u8; N], Error> {
        let mut bytes = [0; N];
        rustix::io::read(&self.term, &mut bytes)?;
        Ok(bytes)
    }

    pub fn new_line(&self) -> Result<(), Error> {
        rustix::io::write(&self.term, "\n".as_bytes())?;
        Ok(())
    }

    pub fn write(&self, n: char) -> Result<(), Error> {
        let mut buf = [0u8; 4];
        let bytes = n.encode_utf8(&mut buf).as_bytes();

        rustix::io::write(&self.term, b"\x1b[1@")?;
        rustix::io::write(&self.term, bytes)?;

        Ok(())
    }

    pub fn write_str(&self, s: &str) -> Result<(), Error> {
        let count = s.chars().count();
        let sequence = format!("\x1b[{}@", count);

        rustix::io::write(&self.term, sequence.as_bytes())?;
        rustix::io::write(&self.term, s.as_bytes())?;

        Ok(())
    }

    pub fn write_with_prompt(&self, prompt: &str, input: String) -> Result<(), Error> {
        let line = format!("\r\x1b[2K{prompt}{input}");
        rustix::io::write(&self.term, line.as_bytes())?;
        Ok(())
    }

    pub fn cursor_left(&self) -> Result<(), Error> {
        rustix::io::write(&self.term, "\x1b[D".as_bytes())?;
        Ok(())
    }

    pub fn cursor_right(&self) -> Result<(), Error> {
        rustix::io::write(&self.term, b"\x1b[C")?;
        Ok(())
    }

    pub fn backspace(&self) -> Result<(), Error> {
        rustix::io::write(&self.term, b"\x08 \x08")?;
        Ok(())
    }

    pub fn delete(&self) -> Result<(), Error> {
        rustix::io::write(&self.term, b"\x1b[P")?;
        Ok(())
    }
}
