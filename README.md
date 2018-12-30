dir-diff
========

A utility that, given two directories, recursively identifies which files are in one directory but not the other. Files are compared by their hash values as calculated by the [XXHash](https://github.com/shepmaster/twox-hash) algorithm.

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
    dir-diff.exe <dir1> <dir2>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

ARGS:
    <dir1>    A directory.
    <dir2>    Another directory.
```
