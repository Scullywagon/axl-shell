use std::{
    fs::{self, OpenOptions},
    io::{BufRead, BufReader, Error, Write},
    path::PathBuf,
};

pub struct History {
    index: usize,
    live_history: Vec<String>,
    history_file: Option<fs::File>,
}

impl History {
    pub fn new(history_dir: Option<PathBuf>) -> Result<Self, Error> {
        let history_file = history_dir
            .map(|path| {
                OpenOptions::new()
                    .create(true)
                    .read(true)
                    .append(true)
                    .open(path.join(".axl_history"))
            })
            .transpose()?;

        let live_history = history_file
            .as_ref()
            .and_then(|file| read_history_from_file(file).ok())
            .unwrap_or_default();

        Ok(Self {
            index: live_history.len(),
            live_history,
            history_file,
        })
    }

    pub fn add(&mut self, input: &str) {
        self.live_history.push(input.to_owned());
        if let Some(file) = &mut self.history_file {
            let _ = writeln!(file, "{input}");
        }
    }

    pub fn reset_index(&mut self) {
        self.index = self.live_history.len()
    }

    pub fn get_next(&mut self) -> Option<&String> {
        if self.index > 0 {
            self.index -= 1;
        }

        self.live_history.get(self.index)
    }

    pub fn get_prev(&mut self) -> Option<&String> {
        if self.index < self.live_history.len() {
            self.index += 1;
        }
        self.live_history.get(self.index)
    }

    pub fn at_end(&self) -> bool {
        self.index == self.live_history.len()
    }
}

fn read_history_from_file(file: &fs::File) -> Result<Vec<String>, Error> {
    let reader = BufReader::new(file);

    let mut history = Vec::new();

    for line in reader.lines() {
        history.push(line?);
    }
    Ok(history)
}

#[cfg(test)]
mod tests {
    use super::History;

    #[test]
    fn add_pushes_to_live_history_when_no_file() {
        let mut history = History {
            index: 0,
            live_history: Vec::new(),
            history_file: None,
        };

        history.add("echo hello");

        assert_eq!(history.get_next(), Some(&"echo hello".to_owned()));
    }
}
