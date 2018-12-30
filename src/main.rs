use std::cmp::Ordering;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use clap::{App, Arg};

#[derive(Clone, Debug)]
struct File {
    path: PathBuf,
    hash: u64,
}

impl Ord for File {
    fn cmp(&self, other: &File) -> Ordering {
        self.hash.cmp(&other.hash)
    }
}

impl PartialOrd for File {
    fn partial_cmp(&self, other: &File) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for File {}

impl PartialEq for File {
    fn eq(&self, other: &File) -> bool {
        self.hash == other.hash
    }
}

use std::hash::Hasher;
use std::io;

struct HashWriter<T: Hasher>(T);

impl<T: Hasher> io::Write for HashWriter<T> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.write(buf);
        Ok(buf.len())
    }

    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        self.write(buf).map(|_| ())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn hash_file(path: &Path) -> u64 {
    if let Ok(file) = std::fs::File::open(path) {
        let mut reader = io::BufReader::new(file);
        let mut hash_writer = HashWriter(twox_hash::XxHash::with_seed(0));

        io::copy(&mut reader, &mut hash_writer).unwrap();
        hash_writer.0.finish()
    } else {
        eprintln!("Could not open {}", path.display());
        0
    }
}

fn file_set(directory: &Path) -> BTreeSet<File> {
    println!(
        "Calculating hashes of files in {} recursively...",
        directory.display()
    );

    walkdir::WalkDir::new(directory)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
        .map(|entry| File {
            path: entry.path().to_path_buf(),
            hash: hash_file(entry.path()),
        })
        .collect()
}

fn main() {
    let matches = App::new("dir-diff")
        .version(env!("CARGO_PKG_VERSION"))
        .arg(
            Arg::with_name("dir1")
                .help("A directory.")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("dir2")
                .help("Another directory.")
                .required(true)
                .index(2),
        )
        .get_matches();

    let dir1 = matches.value_of("dir1").map(Path::new).unwrap();
    let dir2 = matches.value_of("dir2").map(Path::new).unwrap();

    let now = std::time::SystemTime::now();
    let dir1_files = file_set(dir1);
    println!(
        "Took {:?} to hash {} files.",
        now.elapsed().unwrap(),
        dir1_files.len()
    );

    let now = std::time::SystemTime::now();
    let dir2_files = file_set(dir2);
    println!(
        "Took {:?} to hash {} files.",
        now.elapsed().unwrap(),
        dir2_files.len()
    );

    println!("Determining symmetric difference between file sets...");
    let unmatched_files: Vec<File> = dir1_files
        .symmetric_difference(&dir2_files)
        .cloned()
        .collect();

    println!("{:?}", unmatched_files);
}
