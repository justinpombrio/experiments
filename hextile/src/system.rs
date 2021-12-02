use std::path;
use std::fs;
use std::io;


pub use std::io::{Read, Write, Seek, BufRead};



//pub use std::io::prelude::*;

pub trait Savable {
    fn save(self, file: &mut WFile);
}

pub trait Loadable {
    fn load(file: &mut RFile) -> Self;
}

// Not thread safe!
#[macro_export]
macro_rules! log {
    ($label:expr) => (
        $crate::system::_internal_log_indent();
        println!("{}", $label);
    );
    ($label:expr, $msg:expr, $($arg:tt)*) => (
        $crate::system::_internal_log_indent();
        println!("{}: {}", $label, &format!($msg, $($arg)*));
    );
}



pub fn log_group<F>(f: F) where F: FnOnce() -> () {
    unsafe {
        DEPTH += 1;
        f();
        DEPTH -= 1;
    }
}

pub struct Dir {
    path: path::PathBuf
}

impl Dir {
    pub fn path(&self) -> &str {
        path_name(&self.path)
    }

    pub fn name(&self) -> &str {
        file_name(&self.path)
    }

    pub fn dir_exists(&self, dirname: &str) -> bool {
        let path = self.path.join(dirname);
        path.is_dir()
    }

    pub fn file_exists(&self, filename: &str) -> bool {
        let path = self.path.join(filename);
        path.is_file()
    }

    pub fn dir(&self, dirname: &str) -> Dir {
        let path = self.path.join(dirname);
        if !path.is_dir() {
            panic!("FS: Failed to open dir. {}", dirname);
        }
        Dir{
            path: path
        }
    }

    pub fn mkdir(&mut self, dirname: &str) -> Dir {
        let path = self.path.join(dirname);
        match fs::create_dir(&path) {
            Ok(()) => {
                log!("Creating dir", "{}", path_name(&path));
                Dir{
                    path: path
                }
            }
            Err(err) => panic!("FS: Failed to create dir. {:?}", err)
        }
    }

    pub fn open(&self, filename: &str) -> RFile {
        let path = self.path.join(filename);
        let file = match fs::File::open(&path) {
            Ok(file) => file,
            Err(err) => panic!("FS: Failed to open file. {}", err)
        };
        let rfile = RFile{
            path: path,
            buffer: io::BufReader::new(file)
        };
        log!("Opening file", "{}", rfile.path());
        rfile
    }

    pub fn contents(&self) -> Vec<String> {
        let mut names = vec!();
        let entries = match fs::read_dir(&self.path) {
            Err(err) => panic!("FS: Failed to get dir contents. {:?}", err),
            Ok(entries) => entries
        };
        for entry in entries {
            match entry {
                Err(err) => panic!("FS: Failed to iterate over dir contents. {:?}", err),
                Ok(entry) => names.push(file_name(&entry.path()).to_string())
            }
        }
        names
    }

    pub fn create(&mut self, filename: &str) -> WFile {
        let path = self.path.join(filename);
        let file = match fs::File::create(&path) {
            Ok(file) => file,
            Err(err) => panic!("Failed to create file: {}", err)
        };
        let wfile = WFile{
            path: path,
            buffer: io::BufWriter::new(file)
        };
        log!("Creating file", "{}", wfile.path());
        wfile
    }

    pub fn clear(&mut self) {
        log!("Clearing", "{}", self.path());
        fs::remove_dir_all(&self.path).unwrap();
        fs::create_dir(&self.path).unwrap();
    }

    pub fn copy_file(&self, filename: &str, dst: &mut Dir) {
        let mut src_file = self.open(filename);
        let mut dst_file = dst.create(filename);
        match io::copy(&mut src_file, &mut dst_file) {
            Ok(_) => (),
            Err(err) => panic!("Failed to copy file: {}", err)
        }
        log!("Copying", "{}", src_file.path());
    }

    pub fn load<T>(&self, filename: &str) -> T where T: Loadable {
        let mut file = self.open(filename);
        T::load(&mut file)
    }

    pub fn save<T>(&mut self, mut t: T, filename: &str) where T: Savable {
        let mut file = self.create(filename);
        t.save(&mut file)
    }
}


pub trait File {
    fn path(&self) -> &str;
    fn name(&self) -> &str;
}


pub struct RFile {
    buffer: io::BufReader<fs::File>,
    path: path::PathBuf
}
impl File for RFile {
    fn path(&self) -> &str {
        path_name(&self.path)
    }

    fn name(&self) -> &str {
        file_name(&self.path)
    }
}
impl io::Read for RFile {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.buffer.read(buf)
    }
}
impl io::Seek for RFile {
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        self.buffer.seek(pos)
    }
}
impl io::BufRead for RFile {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        self.buffer.fill_buf()
    }

    fn consume(&mut self, amt: usize) {
        self.buffer.consume(amt)
    }
}


pub struct WFile {
    buffer: io::BufWriter<fs::File>,
    path: path::PathBuf
}
impl File for WFile {
    fn path(&self) -> &str {
        path_name(&self.path)
    }

    fn name(&self) -> &str {
        file_name(&self.path)
    }
}
impl io::Write for WFile {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.buffer.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.buffer.flush()
    }
}
impl io::Seek for WFile {
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        self.buffer.seek(pos)
    }
}



pub fn unsafe_cur_dir() -> Dir {
    Dir{
        path: path::PathBuf::from("")
    }
}




// Internals //

fn path_name(path: &path::PathBuf) -> &str {
    match path.to_str() {
        Some(name) => name,
        None => panic!("FS: Non-unicode path name {:?}", path)
    }
}

fn file_name(path: &path::PathBuf) -> &str {
    match path.file_name() {
        Some(name) => match name.to_str() {
            Some(name) => name,
            None => panic!("FS: Non-unicode path name {:?}", path)
        },
        None => panic!("FS: Unexpected error in file_name. {:?}", path)
    }
}

pub fn _internal_log_indent() {
    unsafe {
        for _ in 0..DEPTH {
            print!("    ");
        }
    }
}

static mut DEPTH: usize = 0;
