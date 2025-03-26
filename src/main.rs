use alquitran::archive::Archive;
use alquitran::header::BLOCK_SIZE;
use alquitran::header::Format;
use alquitran::issues::eprint_issues;
use std::env;
use std::fs::File;
use std::io;
use std::io::BufReader;
use std::io::Result;
use std::process::exit;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() > 2 {
        eprintln!("usage: alquitran [file.tar]");
        exit(1);
    }
    let mut archive = if args.len() < 2 {
        Archive::new(Box::new(io::stdin()))
    } else {
        let file = File::open(&args[1])?;
        let reader = BufReader::with_capacity(512, file);
        Archive::new(Box::new(reader))
    };
    let result = archive.lint()?;

    if let Some(d) = &result.dump {
        eprint_bytes(&d.bytes, &d.marks, d.offset);
    }
    eprint_issues(&result.issues);
    for path in result.duplicated_paths.iter() {
        eprintln!("=> Multiple entries for path '{}'.", path);
    }

    let format = match result.format {
        Some(f) => match f {
            Format::Gnu => "gnu",
            Format::Pax => "pax",
            Format::Ustar => "ustar",
            Format::V7 => "v7",
        },
        None => "unknown",
    };
    println!("Detected format: {}", format);

    if result.is_portable() {
        println!("No issues found.");
        return Ok(());
    }
    exit(1);
}

fn eprint_bytes(bytes: &[u8], marks: &[u8], offset: usize) {
    for n in 0..32 {
        eprint!("{:08x}: ", n * 16 + offset * BLOCK_SIZE);
        for b in 0..16 {
            let pos: usize = n * 16 + b;
            let u = bytes[pos];
            if marks[pos] == 0 {
                eprint!("{:02x}", u);
            } else if marks[pos] == 1 {
                eprint!("\x1b[0;33m{:02x}\x1b[0m", u);
            } else {
                eprint!("\x1b[0;31m{:02x}\x1b[0m", u);
            }
            if b % 2 == 1 {
                eprint!(" ");
            }
        }
        eprint!(" ");
        for b in 0..16 {
            let pos = n * 16 + b;
            let u = bytes[pos];
            let c = if (32..128).contains(&u) {
                u as char
            } else {
                '.'
            };
            if marks[pos] == 0 {
                eprint!("{}", c);
            } else if marks[pos] == 1 {
                eprint!("\x1b[0;33m{}\x1b[0m", c);
            } else {
                eprint!("\x1b[0;31m{}\x1b[0m", c);
            }
        }
        eprintln!();
    }
}
