use clap::{App, Arg};
use encoding::all::{UTF_16BE, UTF_16LE, UTF_8};
use encoding::{decode, DecoderTrap, EncoderTrap, EncodingRef};
use log::{debug, info};
use simplelog::*;
use std::borrow::Cow;
use std::fs;
use std::io::{self, Read, Write};

use newline_converter::{dos2unix, unix2dos};

mod errors;
use errors::RncError;

fn main() -> Result<(), RncError> {
    let matches = App::new("rnc")
        .version("0.1.1")
        .about("Newline byte(s) converter")
        .arg(Arg::with_name("FILE")
            .help("Sets the input file to use. If not set, processes stdin to stdout")
            .takes_value(true)
            .multiple(true)
        )
        .arg(Arg::with_name("dos2unix")
            .long("dos2unix")
            .required_unless("unix2dos")
            .conflicts_with("unix2dos")
            .help("Convert DOS line endings to Unix (\\r\\n -> \\n)")
        )
        .arg(Arg::with_name("unix2dos")
            .long("unix2dos")
            .required_unless("dos2unix")
            .help("Convert Unix line endings to DOS (\\n -> \\r\\n)")
        )
        .arg(Arg::with_name("OUT")
            .short("o")
            .long("output")
            .takes_value(true)
            .help("Write to OUT instead of FILE or stdout. Can only be used if FILE is specified just once")
        )
        .arg(Arg::with_name("ENCODE")
            .short("e")
            .long("encode")
            .help("Encode output in given encoding")
            .takes_value(true)
            .possible_values(&["utf8", "utf16", "utf16be"])
        )
        .arg(Arg::with_name("FORCE")
            .short("f")
            .long("force")
            .help("Don't omit binary files")
        )
        .arg(Arg::with_name("verbose")
            .short("v")
            .long("verbose")
            .help("Be verbose about the operations")
        )
        .arg(Arg::with_name("debug")
            .short("d")
            .long("debug")
            .help("Print out debug info")
        )
        .get_matches();

    let verbose = matches.is_present("verbose");
    let debug = matches.is_present("debug");
    if debug {
        SimpleLogger::init(LevelFilter::Debug, Config::default()).expect("could not init logger");
    } else if verbose {
        SimpleLogger::init(LevelFilter::Info, Config::default()).expect("could not init logger");
    } else {
        SimpleLogger::init(LevelFilter::Off, Config::default()).expect("could not init logger");
    }

    let encode: Option<EncodingRef> = matches.value_of("ENCODE").map(|x| match x {
        "utf16" => UTF_16LE as EncodingRef,
        "utf16be" => UTF_16BE as EncodingRef,
        _ => UTF_8 as EncodingRef,
    });

    #[allow(clippy::type_complexity)]
    let conv: Box<dyn Fn(&str) -> Cow<str>> = if matches.is_present("dos2unix") {
        Box::new(dos2unix)
    } else if matches.is_present("unix2dos") {
        Box::new(unix2dos)
    } else {
        unreachable!()
    };

    if matches.is_present("OUT")
        && matches.is_present("FILE")
        && matches.values_of("FILE").unwrap().len() > 1
    {
        return Err(RncError::Io(io::Error::new(
            io::ErrorKind::InvalidInput,
            "OUT cannot be used with multiple FILEs",
        )));
    }

    let output = matches.value_of("OUT");

    if encode.is_some() {
        debug!("Forced output encoding: {}", encode.unwrap().name());
    }

    if let Some(filenames) = matches.values_of("FILE") {
        for f in filenames {
            if verbose {
                println!("Processing {} ", f);
            }
            let o = output.unwrap_or(f);
            let r = process_file(f, o, &conv, matches.is_present("FORCE"), encode)?;
            let FileProcessingResult(processed, read, write) = r;
            if processed {
                info!("{}: {} bytes read. {} bytes written", f, read, write);
            } else {
                info!("{}: Skipped binary file", f);
            }
        }
    } else {
        process_stdio(conv, output, encode)?;
    }

    Ok(())
}

fn process_stdio<F>(
    conv: F,
    outfile: Option<&str>,
    encode: Option<EncodingRef>,
) -> Result<(), RncError>
where
    F: Fn(&str) -> std::borrow::Cow<str>,
{
    let stdin = io::stdin();
    let mut out: Box<dyn Write> = match outfile {
        Some(f) => Box::new(fs::File::create(f)?),
        None => Box::new(io::stdout()),
    };
    let mut buffer: Vec<u8> = Vec::new();
    stdin.lock().read_to_end(&mut buffer)?;
    process(
        &buffer,
        conv,
        encode.or(Some(UTF_8 as EncodingRef)),
        &mut out,
    )?;

    Ok(())
}

struct FileProcessingResult(bool, usize, usize);

fn process_file<F>(
    filename: &str,
    out: &str,
    conv: F,
    force_binary: bool,
    encode: Option<EncodingRef>,
) -> Result<FileProcessingResult, RncError>
where
    F: Fn(&str) -> std::borrow::Cow<str>,
{
    let content = fs::read(filename)?;

    let binary = content_inspector::inspect(&content).is_binary();

    if binary && !force_binary {
        return Ok(FileProcessingResult(false, content.len(), 0));
    }

    let mut f = fs::File::create(out)?;
    let outlen = process(&content, conv, encode, &mut f)?;

    Ok(FileProcessingResult(true, content.len(), outlen))
}

fn process<F, O>(
    input: &[u8],
    conv: F,
    encoding: Option<EncodingRef>,
    output: &mut O,
) -> Result<usize, RncError>
where
    F: Fn(&str) -> std::borrow::Cow<str>,
    O: Write,
{
    let (decoding_result, detected_encoding) = decode(input, DecoderTrap::Replace, UTF_8);
    debug!("Detected encoding: {}", detected_encoding.name());
    let as_string = decoding_result?;
    let converted = conv(&as_string);
    let encode_with = encoding.unwrap_or(detected_encoding);
    let encoded = encode_with.encode(&converted, EncoderTrap::Replace)?;
    let bom: Vec<u8> = match encode_with.name() {
        "utf-16le" => vec![0xFF, 0xFE],
        "utf-16be" => vec![0xFE, 0xFF],
        _ => vec![],
    };
    output.write_all(&bom)?;
    output.write_all(&encoded)?;
    output.flush()?;

    Ok(encoded.len())
}
