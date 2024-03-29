# Rust Newline Converter (`rnc`)
A "dos2unix lookalike" written in Rust

[![Build Status](https://dev.azure.com/michal0805/rnc/_apis/build/status/spitfire05.rnc?branchName=master)](https://dev.azure.com/michal0805/rnc/_build/latest?definitionId=1&branchName=master) ![Crates.io](https://img.shields.io/crates/v/rust-newline-converter-cli)

The motivation to write this crate had two main pivot points:
* My "learning Rust" project
* `dos2unix` being either not available or bugged on some weird platforms, like Android shell

However, similarities to `dos2unix` are in functionality only, the CLI interface was *not* designed to be similar to the one of `dos2unix` in any way.

## Tool usage
```
rnc 0.1
Newline byte(s) converter

USAGE:
    rnc.exe [FLAGS] [OPTIONS] --dos2unix --unix2dos [FILE]...

FLAGS:
    -f, --force       Don't omit binary files
    -d, --debug       Print out debug info
        --dos2unix    Convert DOS line endings to Unix (\r\n -> \n)
    -h, --help        Prints help information
        --unix2dos    Convert Unix line endings to DOS (\n -> \r\n)
    -V, --version     Prints version information
    -v, --verbose     Be verbose about the operations

OPTIONS:
    -e, --encode <ENCODE>    Encode output in given encoding [possible values: utf8, utf16, utf16be]
    -o, --output <OUT>       Write to OUT instead of FILE or stdout. Can only be used if FILE is specified just once

ARGS:
    <FILE>...    Sets the input file to use. If not set, processes stdin to stdout
```

## Conversion caveats
`rnc` respects the valid newline character(s) of the input file. That means, if you use `--unix2dos` and there's a lone `\r` in the input buffer, it will *not* be converted to `\r\n`, as it is not valid newline sequence.

In future there might be an option to override this behavior.

## Performance
One of the main developement goals, was to achieve conversion times not worse that the ones of `dos2unix`. On Linux host, the performance is roughly similar (about 0.1s difference when converting ~100MB file), while on windows `rnc` is twice as fast as `dos2unix` (version downloaded from https://sourceforge.net/projects/dos2unix/).

## MSRV
Minimum Supported Rust Version is `1.40.0`.
