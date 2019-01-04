use std::cmp::Ordering;
use std::collections::BTreeSet;
use std::iter::FromIterator;
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

fn file_collection<T: FromIterator<File>>(directory: &Path) -> T {
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

fn find_empty_dirs(directory: &Path) -> Vec<PathBuf> {
    walkdir::WalkDir::new(directory)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| is_empty_dir(e.path()))
        .map(|e| e.path().to_path_buf())
        .collect()
}

fn is_empty_dir(path: &Path) -> bool {
    path.is_dir() && std::fs::read_dir(path).map(|i| i.count()).unwrap_or(0) == 0
}

fn parent_dir_is_date(path: &Path) -> bool {
    path.parent()
        .unwrap()
        .file_name()
        .unwrap()
        .to_string_lossy()
        .starts_with("20")
}

fn delete_duplicates(paths: &[PathBuf]) {
    let in_album = paths.into_iter().any(|path| !parent_dir_is_date(path));

    if in_album && paths.len() > 1 {
        paths.into_iter().for_each(|path| {
            if parent_dir_is_date(path) {
                println!("Deleting {}", path.display());
                std::fs::remove_file(path).unwrap();
            }
        })
    }
}

fn diff_directories(dir1: &Path, dir2: &Path) -> Vec<PathBuf> {
    let now = std::time::SystemTime::now();
    let dir1_files: BTreeSet<File> = file_collection(&dir1);
    println!(
        "Took {:?} to hash {} files.",
        now.elapsed().unwrap(),
        dir1_files.len()
    );

    let now = std::time::SystemTime::now();
    let dir2_files: BTreeSet<File> = file_collection(dir2);
    println!(
        "Took {:?} to hash {} files.",
        now.elapsed().unwrap(),
        dir2_files.len()
    );

    println!("Determining symmetric difference between file sets...");
    dir1_files
        .symmetric_difference(&dir2_files)
        .map(|f| f.path.clone())
        .collect()
}

fn find_and_delete_duplicates(directory: &Path) {
    let now = std::time::SystemTime::now();
    let mut files: Vec<File> = file_collection(&directory);
    println!(
        "Took {:?} to hash {} files.",
        now.elapsed().unwrap(),
        files.len()
    );

    files.sort_unstable();

    let mut last_hash: Option<u64> = None;
    let mut current_run: Vec<PathBuf> = Vec::new();
    for file in files {
        match last_hash {
            Some(h) if h != file.hash => {
                delete_duplicates(&current_run);
                current_run.clear();
                last_hash = Some(file.hash);
            }
            None => last_hash = Some(file.hash),
            _ => {}
        }
        current_run.push(file.path.clone());
    }

    let now = std::time::SystemTime::now();
    let files = find_empty_dirs(&directory);
    println!("Took {:?} to find empty dirs.", now.elapsed().unwrap());

    files
        .into_iter()
        .for_each(|path| std::fs::remove_dir(path).unwrap());
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
                .help(
                    "Another directory. If specified, the utility prints a \
                     list of files that are unique to dir1 or dir2 according \
                     to their hashes. If unspecified, the utility deletes \
                     duplicate files and empty directories in dir1.",
                )
                .index(2),
        )
        .get_matches();

    let dir1 = matches.value_of("dir1").map(Path::new).unwrap();
    let dir2_option = matches.value_of("dir1").map(Path::new);

    if let Some(dir2) = dir2_option {
        println!(
            "Diffing the directories {} and {}",
            dir1.display(),
            dir2.display()
        );
        let unmatched_files = diff_directories(dir1, dir2);
        println!("{:?}", unmatched_files);
    } else {
        println!(
            "Removing duplicate files and empty directories in {}",
            dir1.display()
        );
        find_and_delete_duplicates(dir1);
    }
}
