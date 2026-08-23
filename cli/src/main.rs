//! `kaz`: pipe data to an honest terminal plot.
//!
//! A thin adapter over the public malevich API — argument parsing, stdin framing,
//! a stream-wiring layer, and calls into presets and grammar. It contains zero
//! rendering logic; if it ever needed a private hook, that would be a gap in the
//! library's public surface, not a place for a back door.

mod args;
mod chart;
mod emit;
mod help;
mod input;
mod live;
mod output;
mod series;
mod time;

use std::io::{self, BufRead, Read, Write};
use std::process::ExitCode;

use args::{Args, Fail, Outcome};

fn main() -> ExitCode {
    match run() {
        Ok(code) => ExitCode::from(code as u8),
        Err(fail) => {
            eprintln!("kaz: {fail}");
            ExitCode::from(2)
        }
    }
}

fn run() -> Result<i32, Fail> {
    match args::parse()? {
        Outcome::Version => {
            println!("kaz {}", env!("CARGO_PKG_VERSION"));
            Ok(0)
        }
        Outcome::Help(topic) => {
            print!("{}", help::text(topic));
            Ok(0)
        }
        Outcome::Run(args) => execute(&args),
    }
}

fn execute(args: &Args) -> Result<i32, Fail> {
    // Prime the color environment before anything reads it (and before any thread).
    output::apply_color(args.color);

    if args.live {
        return match live::run(args) {
            Ok(()) => Ok(0),
            Err(error) => Err(Fail(format!("live: {error}"))),
        };
    }

    let raw = read_input(args)?;
    let mut table = input::frame(&raw, args.delimiter, args.header);
    if let Some(selectors) = &args.cols {
        table = input::select(&table, selectors).map_err(Fail)?;
    }
    // `--by` pulls its column out of the table: the remaining columns are the
    // chart's data, the extracted one is the categorical channel.
    let mut categories = None;
    if let Some(selector) = &args.by {
        let index = input::column_index(&table, selector).map_err(Fail)?;
        categories = Some(input::string_column(&table, index));
        let keep: Vec<String> = (0..table.width())
            .filter(|&column| column != index)
            .map(|column| column.to_string())
            .collect();
        table = input::select(&table, &keep).map_err(Fail)?;
    }
    let built = chart::build(args, &table, categories.as_deref());

    if args.emit_code {
        let program = emit::program(args, &table, categories.as_deref());
        print!("{program}");
        return Ok(0);
    }

    match output::emit(args, &built) {
        Ok(code) => Ok(code),
        // A closed downstream (`kaz … | head`) is a clean stop, not an error.
        Err(output::EmitError::Io(error)) if error.kind() == io::ErrorKind::BrokenPipe => Ok(0),
        Err(output::EmitError::Io(error)) => Err(Fail(format!("write failed: {error}"))),
        Err(output::EmitError::Render(error)) => Err(Fail(format!("render failed: {error}"))),
    }
}

/// Reads the whole input — a positional file, or stdin. With `-O`, the input is
/// echoed to stdout as it is read — line-buffered for stdin, so a downstream
/// consumer starts receiving before the upstream finishes.
fn read_input(args: &Args) -> Result<String, Fail> {
    match &args.input {
        Some(path) => {
            let text = std::fs::read_to_string(path)
                .map_err(|error| Fail(format!("{}: {error}", path.display())))?;
            if args.passthrough {
                echo(text.as_bytes())?;
            }
            Ok(text)
        }
        None if args.passthrough => tee_stdin(),
        None => {
            let mut text = String::new();
            io::stdin()
                .read_to_string(&mut text)
                .map_err(|error| Fail(format!("reading stdin: {error}")))?;
            Ok(text)
        }
    }
}

/// Reads stdin to EOF for the plot, echoing every line to stdout the moment it
/// arrives — the mid-pipeline contract: data flows on while kaz is still reading.
fn tee_stdin() -> Result<String, Fail> {
    let mut stdin = io::stdin().lock();
    let mut text = String::new();
    let mut echoing = true;
    loop {
        let line_start = text.len();
        match stdin.read_line(&mut text) {
            Ok(0) => return Ok(text),
            Ok(_) if echoing => echoing = echo(&text.as_bytes()[line_start..])?,
            Ok(_) => {}
            Err(error) => return Err(Fail(format!("reading stdin: {error}"))),
        }
    }
}

/// Writes one echo chunk to stdout, flushed. Returns whether stdout is still
/// open: a downstream that closed early (`kaz … -O | head`) ends the echo — the
/// plot still renders from everything read.
fn echo(bytes: &[u8]) -> Result<bool, Fail> {
    let mut stdout = io::stdout().lock();
    match stdout.write_all(bytes).and_then(|()| stdout.flush()) {
        Ok(()) => Ok(true),
        Err(error) if error.kind() == io::ErrorKind::BrokenPipe => Ok(false),
        Err(error) => Err(Fail(format!("write failed: {error}"))),
    }
}
