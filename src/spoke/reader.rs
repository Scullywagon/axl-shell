use super::ascii;
use super::history::History;
use super::input::Input;
use super::terminal::Terminal;

use std::io::{self, Error, Write};
use std::os::fd::AsFd;
use std::path::PathBuf;

#[derive(Debug, PartialEq)]
pub enum Line {
    Res(String),
    EoF,
    Interrupt,
}

pub struct Reader<T: AsFd> {
    terminal: Terminal<T>,

    input: Input,
    history: Option<History>,
}

impl<T> Reader<T>
where
    T: AsFd,
{
    pub fn new(term: T, history_dir: Option<PathBuf>) -> Result<Self, Error> {
        let history = History::new(history_dir)?;
        Ok(Self {
            terminal: Terminal::new(term)?,

            input: Input::new(),
            history: Some(history),
        })
    }

    pub fn read(&mut self, prompt: &str) -> Result<Line, Error> {
        self.terminal.write_str(prompt)?;

        loop {
            let byte = self.terminal.read_bytes::<1>()?;

            match byte[0] {
                ascii::NEW_LINE | ascii::RETURN => {
                    self.terminal.new_line()?;
                    self.terminal.reset()?;
                    return Ok(Line::Res(self.input.get_active()));
                }

                ascii::END_OF_TEXT => {
                    self.terminal.write_str("^C")?;
                    self.terminal.new_line()?;
                    return Ok(Line::Interrupt);
                }

                // same as Ctrl-D
                ascii::END_OF_FILE if self.input.is_empty() => {
                    self.terminal.new_line()?;
                    return Ok(Line::EoF);
                }
                // backspace
                ascii::BACKSPACE if !self.input.is_empty() => {
                    self.terminal.backspace()?;
                    self.input.backspace();
                    continue;
                }

                ascii::ESCAPE => self.match_escape_bytes(prompt)?,

                byte if byte.is_ascii() && !byte.is_ascii_control() => {
                    let character = byte as char;
                    self.input.push(character);
                    self.terminal.write(character)?;
                }
                _ => {}
            }
        }
    }

    fn match_escape_bytes(&mut self, prompt: &str) -> Result<(), Error> {
        let bytes = self.terminal.read_bytes::<2>()?;

        match bytes {
            ascii::UP => match &mut self.history {
                None => {}
                Some(history) => {
                    if history.at_end() {
                        self.input.save_active_as_draft();
                    }

                    if let Some(entry) = history.get_next() {
                        self.input.set_active(entry.clone());
                    }
                    self.terminal
                        .write_with_prompt(prompt, self.input.get_active())?;
                    return Ok(());
                }
            },
            ascii::DOWN => match &mut self.history {
                None => {}
                Some(history) => {
                    match history.get_prev() {
                        Some(entry) => {
                            self.input.set_active(entry.clone());
                        }
                        None => {
                            self.input.save_active_as_draft();
                        }
                    }
                    self.terminal
                        .write_with_prompt(prompt, self.input.get_active())?;
                    return Ok(());
                }
            },
            ascii::LEFT => {
                if self.input.cursor_left() {
                    self.terminal.cursor_left()?;
                }
                return Ok(());
            }
            ascii::RIGHT => {
                if self.input.cursor_right() {
                    self.terminal.cursor_right()?;
                }
                return Ok(());
            }
            _ => {}
        };

        let next_byte = self.terminal.read_bytes::<1>()?;
        let long_bytes = [bytes[0], bytes[1], next_byte[0]];

        match long_bytes {
            ascii::DELETE if !self.input.is_empty() => {
                self.input.delete();
                self.terminal.delete()?;
            }
            _ => {}
        }

        Ok(())
    }
}

impl<T> Drop for Reader<T>
where
    T: AsFd,
{
    fn drop(&mut self) {
        let _ = self.terminal.reset();
    }
}
