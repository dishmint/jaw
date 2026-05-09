use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::ExitCode;

mod export;
mod lang;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(msg) => {
            eprintln!("{}", msg);
            ExitCode::FAILURE
        }
    }
}

fn run(args: Vec<String>) -> Result<(), String> {
    let mut iter = args.into_iter();
    let sub = iter.next();
    match sub.as_deref() {
        Some("export") => cmd_export(iter.collect()),
        Some("--version") | Some("-V") => {
            println!("{}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        Some("--help") | Some("-h") | Some("help") | None => {
            print_usage();
            Ok(())
        }
        Some(other) => Err(format!(
            "unknown subcommand: {}\n\n{}",
            other,
            usage_text()
        )),
    }
}

fn cmd_export(args: Vec<String>) -> Result<(), String> {
    let mut lang_name: Option<String> = None;
    let mut output: Option<PathBuf> = None;
    let mut input: Option<PathBuf> = None;
    let mut iter = args.into_iter();

    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--lang" | "-l" => {
                lang_name = Some(
                    iter.next()
                        .ok_or_else(|| "--lang requires a value".to_string())?,
                );
            }
            "--output" | "-o" => {
                output = Some(PathBuf::from(
                    iter.next()
                        .ok_or_else(|| "--output requires a value".to_string())?,
                ));
            }
            "--help" | "-h" => {
                print_export_usage();
                return Ok(());
            }
            s if s.starts_with('-') => {
                return Err(format!("unknown flag for export: {}", s));
            }
            _ => {
                if input.is_some() {
                    return Err("export takes a single input file".to_string());
                }
                input = Some(PathBuf::from(arg));
            }
        }
    }

    let lang_name = lang_name.ok_or_else(|| {
        format!(
            "--lang is required (one of: {})",
            lang::known_languages().join(", ")
        )
    })?;
    let lang = lang::lookup(&lang_name).ok_or_else(|| {
        format!(
            "unknown language: {}\nsupported: {}",
            lang_name,
            lang::known_languages().join(", ")
        )
    })?;
    let input = input.ok_or_else(|| "missing input file".to_string())?;

    let source = fs::read_to_string(&input)
        .map_err(|e| format!("could not read {}: {}", input.display(), e))?;
    let result = export::export(&source, lang);

    match output {
        Some(path) => fs::write(&path, result)
            .map_err(|e| format!("could not write {}: {}", path.display(), e))?,
        None => {
            let stdout = io::stdout();
            let mut h = stdout.lock();
            h.write_all(result.as_bytes())
                .map_err(|e| format!("write to stdout failed: {}", e))?;
        }
    }
    Ok(())
}

fn print_usage() {
    println!("{}", usage_text());
}

fn usage_text() -> String {
    format!(
        "jaw {}\n\n\
         USAGE:\n    \
         jaw <SUBCOMMAND>\n\n\
         SUBCOMMANDS:\n    \
         export    Export a .jaw file as comments in a target language\n    \
         help      Print this help\n",
        env!("CARGO_PKG_VERSION")
    )
}

fn print_export_usage() {
    println!(
        "jaw export — convert a .jaw file to target-language comments\n\n\
         USAGE:\n    \
         jaw export --lang LANG [--output PATH] FILE\n\n\
         OPTIONS:\n    \
         -l, --lang LANG       target language ({})\n    \
         -o, --output PATH     write to PATH (default: stdout)\n    \
         -h, --help            print this help\n",
        lang::known_languages().join(", ")
    );
}
