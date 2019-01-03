dir-diff
========

A utility that, given two directories, recursively identifies which files are in
one directory but not the other. Files are compared by their hash values as
calculated by the [XXHash](https://github.com/shepmaster/twox-hash) algorithm.

If only one directory is given, it recursively finds and deletes duplicate files
(identified by their hash values) and empty directories in that directory. The
utility prefers to retain non-date-format directories (any that don't have
names starting `20`, for simplicity).

## Build

Install Rust 1.31+ and run

```
cargo build --release
```

## Run

```
./dir-diff.exe --help
dir-diff 0.1.0

USAGE:
    dir-diff.exe <dir1> [dir2]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

ARGS:
    <dir1>    A directory.
    <dir2>    Another directory. If specified, the utility prints a list of files that are unique to dir1 or dir2
              according to their hashes. If unspecified, the utility deletes duplicate files and empty directories
              in dir1.
```
