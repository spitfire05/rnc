# rnc
Rust Newline Converter - a "dos2unix lookalike" written in Rust

[![Build Status](https://dev.azure.com/michal0805/rnc/_apis/build/status/spitfire05.rnc?branchName=master)](https://dev.azure.com/michal0805/rnc/_build/latest?definitionId=1&branchName=master)

The motivation to write this crate had two main pivot points:
* My "learning Rust" project
* `dos2unix` being either not available or bugged on some weird platforms, like Android shell

However, similarities to `dos2unix` are in functionality only, the CLI interface was *not* designed to be similar to the one of `dos2unix` in any way.

## Crate contents
`rnc` create provides both library and CLI tool to convert the newline characters in buffers.

## Tool usage
```
rnc 0.1
Converts line endings

USAGE:
    rnc.exe [FLAGS] [OPTIONS] --dos2unix --unix2dos [FILE]...

FLAGS:
        --dos2unix    Convert DOS line endings to Unix (\r\n -> \n)
    -h, --help        Prints help information
        --unix2dos    Convert Unix line endings to DOS (\n -> \r\n)
    -V, --version     Prints version information
    -v, --verbose     Be verbose about the operations

OPTIONS:
    -e, --encoding <ENCODING>    Treat input as ENCODING (default is utf8 for stdin, and an educated guess for files)
                                 [possible values: utf8, utf16le, utf16be, utf32le, utf32be]
    -o, --output <OUT>           Write to OUT instead of FILE or stdout. Can only be used if FILE is specified just once

ARGS:
    <FILE>...    Sets the input file to use. If not set, processes stdin to stdout
```
