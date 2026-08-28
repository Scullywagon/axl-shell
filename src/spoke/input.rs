use std::fmt::{Error, Write};

pub struct Input {
    draft: Option<String>,
    active: String,
    cursor: usize,
}

impl Input {
    pub fn new() -> Self {
        Self {
            draft: None,
            active: String::new(),
            cursor: 0,
        }
    }
    pub fn push(&mut self, character: char) {
        self.active.insert(self.cursor, character);
        self.cursor += 1;
    }

    pub fn push_string(&mut self, string: &str) -> Result<(), Error> {
        self.active.insert_str(self.cursor, string);
        self.cursor += string.len();
        Ok(())
    }

    pub fn is_empty(&self) -> bool {
        self.active.is_empty()
    }

    pub fn get_cursor(&self) -> usize {
        self.cursor
    }

    pub fn backspace(&mut self) {
        self.active.remove(self.cursor - 1);
        self.cursor -= 1;
    }

    pub fn delete(&mut self) {
        if self.cursor >= self.active.len() {
            return;
        }
        self.active.remove(self.cursor);
    }

    pub fn cursor_left(&mut self) -> bool {
        if self.cursor == 0 {
            return false
        }
        self.cursor -= 1;
        true
    }

    pub fn cursor_right(&mut self) -> bool {
        if self.cursor < self.active.len() {
            self.cursor += 1;
            return true
        }
        false
    }

    pub fn has_draft(&self) -> bool {
        self.draft.is_some()
    }

    pub fn save_active_as_draft(&mut self) {
        self.draft = Some(self.active.clone());
    }

    pub fn draft_as_active(&mut self) {
        self.active = self.draft.take().unwrap_or_default();
    }

    pub fn set_active(&mut self, actv: String) {
        self.active = actv;
    }

    pub fn get_active(&self) -> String {
        self.active.clone()
    }
}
