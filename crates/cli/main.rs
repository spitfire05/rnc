use clap::{App, Arg};
use content_inspector::inspect;
use std::error::Error;
use std::fs;
use std::io::Write;
use std::{
    borrow::Cow,
    io::{self, Read},
};

use newline_converter::{dos2unix, unix2dos};

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
            .help("Convert Unix line endings to DOS (\\n -> \\r\\n)")
        )
        .arg(Arg::with_name("OUT")
            .short("o")
            .long("output")
            .takes_value(true)
            .help("Write to OUT instead of FILE or stdout. Can only be used if FILE is specified just once")
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
        .get_matches();

    let verbose = matches.is_present("verbose");

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
        return Err(Box::new(io::Error::new(
            io::ErrorKind::InvalidInput,
            "OUT cannot be used with multiple FILEs",
        )));
    }

    let output = matches.value_of("OUT");

    if let Some(filenames) = matches.values_of("FILE") {
        for f in filenames {
            if verbose {
                println!("Processing {} ", f);
            }
            let o = output.unwrap_or(f);
            let r = process_file(f, o, &conv, matches.is_present("FORCE"))?;
            if verbose {
                let FileProcessingResult(processed, read, write) = r;
                if processed {
                    println!("{}: {} bytes read. {} bytes written", f, read, write);
                } else {
                    println!("{}: Skipped binary file", f);
                }
            }
        }
    } else {
        process_stdio(conv, output)?;
    }

    Ok(())
}

fn process_stdio<F>(conv: F, outfile: Option<&str>) -> Result<(), Box<dyn Error>>
where
    F: Fn(&str) -> std::borrow::Cow<str>,
{
    let stdin = io::stdin();
    let mut out: Box<dyn Write> = match outfile {
        Some(f) => Box::new(fs::File::create(f)?),
        None => Box::new(io::stdout()),
    };
    let mut buffer = String::new();
    stdin.lock().read_to_string(&mut buffer)?;
    let converted = conv(&buffer);
    out.write_all(converted.as_bytes())?;
    out.flush()?;

    Ok(())
}

struct FileProcessingResult(bool, usize, usize);

fn process_file<F>(
    filename: &str,
    out: &str,
    conv: F,
    force_binary: bool,
) -> Result<FileProcessingResult, std::io::Error>
where
    F: Fn(&str) -> std::borrow::Cow<str>,
{
    let content = fs::read(filename)?;

    if !force_binary && inspect(&content).is_binary() {
        return Ok(FileProcessingResult(false, content.len(), 0));
    }

    let as_string = String::from_utf8_lossy(&content);

    let converted = conv(&as_string);

    fs::write(out, converted.as_ref())?;

    Ok(FileProcessingResult(true, content.len(), converted.len()))
}
