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
        .author("Michal Borejszo")
        .about("Converts line endings")
        .arg(Arg::with_name("FILE")
            .help("Sets the input file to use. If not set, processes stdin to stdout")
            .takes_value(true)
            .multiple(true)
        )
        .arg(Arg::with_name("dos2unix")
            .long("dos2unix")
            .required_unless("unix2dos")
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
        panic!("Parser failure");
    }



    if matches.is_present("OUT") && matches.is_present("FILE") && matches.values_of("FILE").unwrap().len() > 1 {
        return Err(Box::new(io::Error::new(io::ErrorKind::InvalidInput, "OUT cannot be used with multipple FILEs")));
    }

    let output = matches.value_of("OUT");

    if let Some(filenames) = matches.values_of("FILE"){
        for f in filenames {
            if verbose {
                println!("Processing {} ", f);
            }
            let o = match output {
                Some(o) => o,
                None => f
            };
            let r = process_file(f, o, &conv)?;
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
        process_stdio(conv, output)?;
    }

    Ok(())
}

fn process_stdio(conv: Conversion, outfile: Option<&str>) -> Result<(), Box<dyn Error>> {
    let stdin = io::stdin();
    let mut out: Box<dyn Write> = match outfile {
        Some(f) => Box::new(fs::File::create(f)?),
        None => Box::new(io::stdout())
    };
    let mut buffer: [u8; 1024] = [0; 1024];
    loop {
        let n = stdin.lock().read(&mut buffer)?;
        if n == 0 {
            break;
        }
        let converted = convert(&buffer[0..n], &conv, 1, &ByteOrder::LittleEndian)?;
        out.write_all(&converted)?;
        out.flush()?;
    }

    Ok(())
}

struct FileProcessingResult (bool, usize, usize);

fn process_file(filename: &str, out: &str, conv: &Conversion) -> Result<FileProcessingResult, Box<dyn Error>> {
    let content = fs::read(filename)?;

    let (process, char_size, order) = match inspect(&content) {
        ContentType::UTF_8 | ContentType::UTF_8_BOM => (true, 1, ByteOrder::LittleEndian),
        ContentType::UTF_16LE => (true, 2, ByteOrder::LittleEndian),
        ContentType::UTF_16BE => (true, 2, ByteOrder::BigEndian),
        ContentType::UTF_32LE => (true, 4, ByteOrder::LittleEndian),
        ContentType::UTF_32BE => (true, 4, ByteOrder::BigEndian),
        ContentType::BINARY => (false, 0, ByteOrder::LittleEndian),
    };

    if !process {
        return Ok(FileProcessingResult(false, content.len(), 0));
    }

    // TODO: determine byte length, skip binary
    let converted = convert(&content, conv, char_size, &order)?;

    fs::write(out, &converted)?;

    Ok(FileProcessingResult(true, content.len(), converted.len()))
}
