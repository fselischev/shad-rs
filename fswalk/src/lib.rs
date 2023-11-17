#![forbid(unsafe_code)]

use std::{
    fs::{self},
    io::{self, Read},
    path::Path,
};

////////////////////////////////////////////////////////////////////////////////

type Callback<'a> = dyn FnMut(&mut Handle) + 'a;

#[derive(Default)]
pub struct Walker<'a> {
    callbacks: Vec<Box<Callback<'a>>>,
}

impl<'a> Walker<'a> {
    pub fn new() -> Self {
        Self {
            callbacks: Vec::new(),
        }
    }

    pub fn add_callback<F>(&mut self, callback: F)
    where
        F: FnMut(&mut Handle) + 'a,
    {
        self.callbacks.push(Box::new(callback));
    }

    pub fn walk<P: AsRef<Path>>(&mut self, path: P) -> io::Result<()> {
        if self.callbacks.is_empty() {
            return Ok(());
        }
        Self::rec_walk(path.as_ref(), self.callbacks.as_mut_slice())
    }

    fn rec_walk(dir: &Path, callbacks: &mut [Box<Callback>]) -> io::Result<()> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            let mut handle = {
                if path.is_file() {
                    Handle::File(FileHandle {
                        path: &path,
                        read: false,
                    })
                } else if path.is_dir() {
                    Handle::Dir(DirHandle {
                        path: &path,
                        descend: false,
                    })
                } else {
                    continue;
                }
            };

            let mut idx = 0;
            for i in 0..callbacks.len() {
                callbacks[i](&mut handle);
                if Self::checked(&mut handle) {
                    if idx < i {
                        callbacks.swap(idx, i);
                    }
                    idx += 1;
                }
            }

            match handle {
                Handle::Dir(dir) => Self::rec_walk(dir.path(), &mut callbacks[0..idx])?,
                Handle::File(file_handle) => {
                    let mut file = fs::File::open(file_handle.path())?;
                    let mut buf = Vec::new();
                    file.read_to_end(&mut buf)?;
                    let mut content_handle = Handle::Content {
                        file_path: file_handle.path(),
                        content: &buf,
                    };
                    for cb in callbacks.iter_mut().take(idx) {
                        cb(&mut content_handle);
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn checked(handle: &mut Handle) -> bool {
        match handle {
            Handle::Dir(dir) => {
                let fl = dir.descend;
                dir.descend = false;
                fl
            }
            Handle::File(file) => {
                let fl = file.read;
                file.read = false;
                fl
            }
            _ => false,
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

pub enum Handle<'a> {
    Dir(DirHandle<'a>),
    File(FileHandle<'a>),
    Content {
        file_path: &'a Path,
        content: &'a [u8],
    },
}

pub struct DirHandle<'a> {
    path: &'a Path,
    descend: bool,
}

impl<'a> DirHandle<'a> {
    pub fn descend(&mut self) {
        self.descend = true;
    }

    pub fn path(&self) -> &Path {
        self.path
    }
}

pub struct FileHandle<'a> {
    path: &'a Path,
    read: bool,
}

impl<'a> FileHandle<'a> {
    pub fn read(&mut self) {
        self.read = true
    }

    pub fn path(&self) -> &Path {
        self.path
    }
}
