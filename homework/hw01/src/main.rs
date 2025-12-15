use std::io::{BufRead, BufReader};

use hw01::take_expr;

fn main() {
    let stdin = BufReader::new(std::io::stdin());
    for line in stdin.lines() {
        let line = line.unwrap();
        let v = line
            .trim()
            .as_bytes()
            .iter()
            .map(|b| *b as char)
            .collect::<Vec<_>>();
        match take_expr(&mut v.as_slice()) {
            Ok(e) => {
                println!("Parsed: {e}");
            }
            Err(e) => {
                eprintln!("Error: {e}");
            }
        }
    }
}
