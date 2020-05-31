# newline-converter
`newline-converter` is a simple library used for converting the newline bytes in buffers between Windows `\r\n` and Unix `\n` style. All I/O data are `u8` buffers.

It mainly serves as a backend for "Rust Newline converter" CLI tool.

## Examples

UTF8 `dos2unix` conversion:

```rust
let dos2unix = Converter::utf8(Conversion::Dos2unix);
assert_eq!(
    dos2unix.convert(b"\nfoo\rbar\r\n").unwrap(),
    b"\nfoo\rbar\n"
);
```

UTF16 Little Endian `unix2dos` conversion:

```rust
let some_string = "foobar\r\n";
let dos2unix = Converter::utf8(Conversion::Dos2unix);
let bytes = dos2unix.convert(some_string.as_bytes()).unwrap();
let new_string = String::from_utf8(bytes).unwrap();
assert_eq!("foobar\n", new_string);
```