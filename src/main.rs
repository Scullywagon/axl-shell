mod spoke;

use std::io::{self, Write};

use crate::spoke::Line;

fn main() -> std::io::Result<()> {
    let prompt = "temp >> ";
    let stdin = io::stdin();
    let mut term = spoke::Reader::new(stdin, dirs::home_dir())?;

    loop {
        match term.read(prompt) {
            Ok(x) => {
                match x {
                    Line::Res(val) => {
                        println!("{val}");
                    }
                    _ => {}
                }
                break;
            }
            _ => {}
        };
    }

    Ok(())
}
