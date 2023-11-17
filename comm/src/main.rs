#![forbid(unsafe_code)]

use std::{
    collections::HashSet,
    env,
    fs::File,
    io::{BufRead, BufReader},
};

fn read_lines(path: &str) -> HashSet<String> {
    let mut set = HashSet::new();
    let file = File::open(path).unwrap();
    let reader = BufReader::new(file);
    for line in reader.lines() {
        set.insert(line.unwrap());
    }
    set
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = env::args().collect::<Vec<String>>();
    let mut first_lines = read_lines(&args[1]);

    let file = std::fs::File::open(&args[2])?;
    let reader = std::io::BufReader::new(file);
    for line in reader.lines() {
        let line = line?;
        if first_lines.contains(&line) {
            first_lines.remove(&line);
            println!("{}", line);
        }
    }

    Ok(())
}
