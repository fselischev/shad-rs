#![forbid(unsafe_code)]

use std::{
    fs::File,
    io::{self, BufRead, BufReader},
    path::{Path, PathBuf},
    sync::mpsc::{self, Sender},
};

use rayon::prelude::*;

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, PartialEq, Eq)]
pub struct Match {
    pub path: PathBuf,
    pub line: String,
    pub line_number: usize,
}

#[derive(Debug)]
pub struct Error {
    pub path: PathBuf,
    pub error: io::Error,
}

pub enum Event {
    Match(Match),
    Error(Error),
}

pub fn run<P: AsRef<Path>>(path: P, pattern: &str) -> Vec<Event> {
    let path = path.as_ref();
    let (sender, receiver) = mpsc::channel();

    if path.is_file() {
        process_file(path, pattern, sender.clone());
    } else if path.is_dir() {
        get_files_in_directory(path)
            .par_iter()
            .for_each(|file| process_file(file, pattern, sender.clone()));
    } else {
        return vec![Event::Error(Error {
            path: path.to_path_buf(),
            error: io::Error::new(io::ErrorKind::Other, "Invalid path"),
        })];
    }

    drop(sender);
    receiver.iter().collect::<Vec<_>>()
}

fn process_file<P: AsRef<Path>>(file_path: P, pattern: &str, sender: Sender<Event>) {
    let file = File::open(&file_path).unwrap();
    let reader = BufReader::new(file);

    for (line_number, line) in reader.lines().enumerate() {
        if let Ok(line) = line {
            if line.contains(pattern) {
                sender
                    .send(Event::Match(Match {
                        path: file_path.as_ref().to_path_buf(),
                        line,
                        line_number: line_number + 1,
                    }))
                    .unwrap();
            }
        }
    }
}

fn get_files_in_directory(directory: &Path) -> Vec<PathBuf> {
    let mut files = vec![];
    visit_dirs(directory, &mut files);
    files
}

fn visit_dirs(dir: &Path, files: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();

        if path.is_dir() {
            visit_dirs(&path, files);
        } else {
            files.push(path);
        }
    }
}
