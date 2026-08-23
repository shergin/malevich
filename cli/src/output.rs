//! Stream wiring: build the frame for the destination, render at the chosen tier,
//! and write — plot to its target, data through to stdout (D-C4, D-C10).

use std::fs::File;
use std::io::{self, IsTerminal, Write};

use malevich::pixel::Capabilities;
use malevich::{Frame, Plot};

use crate::args::{Args, CharsetChoice, ColorChoice, Output, PixelsChoice};
use crate::chart::Built;

/// A destination write or bounded-render failure.
#[derive(Debug)]
pub enum EmitError {
    Io(io::Error),
    Render(malevich::Error),
}

impl From<io::Error> for EmitError {
    fn from(error: io::Error) -> EmitError {
        EmitError::Io(error)
    }
}

impl From<malevich::Error> for EmitError {
    fn from(error: malevich::Error) -> EmitError {
        EmitError::Render(error)
    }
}

/// Translates the `--color` escape hatch onto the environment variables whose
/// precedence malevich already documents (`NO_COLOR` > `CLICOLOR_FORCE` > tty),
/// so [`Frame::detect_for`] stays the single source of truth for color tiers —
/// no detection logic is duplicated here.
///
/// Must run before any thread is spawned.
pub fn apply_color(choice: ColorChoice) {
    // SAFETY: called once at startup on the main thread, before the program
    // spawns any thread or otherwise reads the environment concurrently.
    match choice {
        ColorChoice::Never => unsafe { std::env::set_var("NO_COLOR", "1") },
        ColorChoice::Always => unsafe {
            // An explicit flag beats ambient configuration: an inherited
            // NO_COLOR must not silence `--color always`.
            std::env::remove_var("NO_COLOR");
            std::env::set_var("CLICOLOR_FORCE", "1");
        },
        ColorChoice::Auto => {}
    }
}

/// Renders and writes the plot to its destination and reports the unparsed-field
/// tally. Returns the process exit code. (`-O` passthrough already happened at
/// read time, line by line — see `main::read_input`.)
pub fn emit(args: &Args, built: &Built<'_>) -> Result<i32, EmitError> {
    match &args.output {
        Output::Stderr => plot_to(io::stderr(), &built.plot, args)?,
        Output::Stdout => plot_to(io::stdout(), &built.plot, args)?,
        Output::File(path) => {
            let file = File::create(path)?;
            plot_to(file, &built.plot, args)?;
        }
    }

    // The tally rides stderr after the plot: data loss must be visible, but a
    // handful of bad log lines is not fatal (D-C6).
    if built.unparsed > 0 && !args.quiet {
        let noun = if built.unparsed == 1 {
            "value"
        } else {
            "values"
        };
        eprintln!("{} {noun} could not be parsed", built.unparsed);
    }
    Ok(0)
}

/// Builds the frame for `dest`, renders the plot at the chosen tier, and writes it.
fn plot_to<W: Write + IsTerminal>(
    mut dest: W,
    plot: &Plot<'_>,
    args: &Args,
) -> Result<(), EmitError> {
    let frame = frame_for(&dest, args);
    let text = render(plot, &frame, args.pixels, &dest)?;
    dest.write_all(text.as_bytes())?;
    dest.write_all(b"\n")?;
    Ok(dest.flush()?)
}

/// The frame for a destination: detection keyed to `dest`, then the `--charset`
/// and `-w`/`-h` overrides. Color comes through [`Frame::detect_for`] because
/// [`apply_color`] has already primed the environment.
pub(crate) fn frame_for<T: IsTerminal>(dest: &T, args: &Args) -> Frame {
    let mut frame = Frame::detect_for(dest);
    if let CharsetChoice::Fixed(charset) = args.charset {
        frame.charset = charset;
    }
    if let Some(width) = args.width {
        frame.width = width;
    }
    if let Some(height) = args.height {
        frame.height = height;
    }
    frame
}

/// Chooses the render tier with capabilities detected for the actual destination.
/// `auto` only attempts pixels for a terminal; `always` also permits a sniffed
/// protocol when writing a pipe or file.
fn render<T: IsTerminal>(
    plot: &Plot<'_>,
    frame: &Frame,
    pixels: PixelsChoice,
    destination: &T,
) -> malevich::Result<String> {
    match pixels {
        PixelsChoice::Never => plot.try_render(frame),
        PixelsChoice::Auto if destination.is_terminal() => {
            plot.try_render_with_capabilities(frame, &Capabilities::detect_for(destination))
        }
        PixelsChoice::Auto => plot.try_render(frame),
        PixelsChoice::Always => {
            plot.try_render_with_capabilities(frame, &Capabilities::detect_for(destination))
        }
    }
}
