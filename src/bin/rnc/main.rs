use std::error::Error;
use std::io::Write;
use content_inspector::{ContentType, inspect};
use std::fs;
use std::io::{self, Read};
use clap::{Arg, App};

use rnc::*;

fn main() -> Result<(), Box<dyn Error>> {
    let matches = App::new("rnc")
        .version("0.1")
        .about("Neline byte(s) converter")
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
            .conflicts_with("dos2unix")
            .help("Convert Unix line endings to DOS (\\n -> \\r\\n)")
        )
        .arg(Arg::with_name("OUT")
            .short("o")
            .long("output")
            .takes_value(true)
            .help("Write to OUT instead of FILE or stdout. Can only be used if FILE is specified just once")
        )
        .arg(Arg::with_name("ENCODING")
            .short("e")
            .long("encoding")
            .help("Treat input as ENCODING (default is utf8 for stdin, and an educated guess for files)")
            .takes_value(true)
            .possible_values(&["utf8", "utf16le", "utf16be", "utf32le", "utf32be"])
        )
        .arg(Arg::with_name("verbose")
            .short("v")
            .long("verbose")
            .help("Be verbose about the operations")
        )
        .get_matches();

    let verbose = matches.is_present("verbose");

    let conv: Conversion;
    if matches.is_present("dos2unix") {
        conv = Conversion::Dos2unix;
    }
    else if matches.is_present("unix2dos") {
        conv = Conversion::Unix2dos;
    }
    else {
        unreachable!()
    }

    if matches.is_present("OUT") && matches.is_present("FILE") && matches.values_of("FILE").unwrap().len() > 1 {
        return Err(Box::new(io::Error::new(io::ErrorKind::InvalidInput, "OUT cannot be used with multipple FILEs")));
    }

    let output = matches.value_of("OUT");

    let (force_char_size, force_order) = match matches.value_of("ENCODING") {
        None => (None, None),
        Some(x) => {
            match x {
                "utf8" => (Some(1), Some(ByteOrder::LittleEndian)),
                "utf16le" => (Some(2), Some(ByteOrder::LittleEndian)),
                "utf16be" => (Some(2), Some(ByteOrder::BigEndian)),
                "utf32le" => (Some(4), Some(ByteOrder::LittleEndian)),
                "utf32be" => (Some(4), Some(ByteOrder::BigEndian)),
                _ => unreachable!()
            }
        }
    };

    if let Some(filenames) = matches.values_of("FILE") {
        for f in filenames {
            if verbose {
                println!("Processing {} ", f);
            }
            let o = match output {
                Some(o) => o,
                None => f
            };
            let r = process_file(f, o, conv, force_char_size, force_order)?;
            if verbose {
                let FileProcessingResult(processed, read, write) = r;
                if processed
                {
                    println!("{}: {} bytes read. {} bytes written", f, read, write);
                }
                else {
                    println!("{}: Skipped binary file", f);
                }
            }
        }
    }
    else {
        process_stdio(conv, output, force_char_size, force_order)?;
    }

    Ok(())
}

fn process_stdio(conv: Conversion, outfile: Option<&str>, force_char_size: Option<usize>, force_order: Option<ByteOrder>) -> Result<(), Box<dyn Error>> {
    let stdin = io::stdin();
    let mut out: Box<dyn Write> = match outfile {
        Some(f) => Box::new(fs::File::create(f)?),
        None => Box::new(io::stdout())
    };
    let converter = Converter::new(conv, force_char_size.unwrap_or(1), force_order.unwrap_or(ByteOrder::LittleEndian));
    let mut buffer: [u8; 1024] = [0; 1024];
    loop {
        let n = stdin.lock().read(&mut buffer)?;
        if n == 0 {
            break;
        }
        let converted = converter.convert(&buffer[0..n])?;
        out.write_all(&converted)?;
        out.flush()?;
    }

    Ok(())
}

struct FileProcessingResult (bool, usize, usize);

fn process_file(filename: &str, out: &str, conv: Conversion, force_char_size: Option<usize>, force_order: Option<ByteOrder>) -> Result<FileProcessingResult, Box<dyn Error>> {
    let content = fs::read(filename)?;

    let converter = match inspect(&content) {
        ContentType::UTF_8 | ContentType::UTF_8_BOM => Some(Converter::new(conv, force_char_size.unwrap_or(1), force_order.unwrap_or(ByteOrder::BigEndian))),
        ContentType::UTF_16LE => Some(Converter::new(conv, force_char_size.unwrap_or(2), force_order.unwrap_or(ByteOrder::LittleEndian))),
        ContentType::UTF_16BE => Some(Converter::new(conv, force_char_size.unwrap_or(2), force_order.unwrap_or(ByteOrder::BigEndian))),
        ContentType::UTF_32LE => Some(Converter::new(conv, force_char_size.unwrap_or(4), force_order.unwrap_or(ByteOrder::LittleEndian))),
        ContentType::UTF_32BE => Some(Converter::new(conv, force_char_size.unwrap_or(4), force_order.unwrap_or(ByteOrder::BigEndian))),
        ContentType::BINARY => None,
    };

    if let Some(c) = converter {
        let converted = c.convert(&content)?;
        fs::write(out, &converted)?;

        Ok(FileProcessingResult(true, content.len(), converted.len()))
    }
    else {
        Ok(FileProcessingResult(false, content.len(), 0))
    }
}   
    
