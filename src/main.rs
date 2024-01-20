//! Command Line interface for xstream
//!
//! This just wraps the xstream lib in argument parsing, and character delimiter conversion, but is
//! much more convenient.
#![warn(clippy::pedantic)]

use clap::builder::NonEmptyStringValueParser;
use clap::{ArgGroup, Parser};
use std::io;
use std::process::Command;
use std::str;
use xstream_util::{Limiting, Rotating};

/// Escape delimiters in a string
///
/// The empty string becomes the null delimiter.
fn unescape_delimiter(char_ref: impl AsRef<str>) -> String {
    let char_str = char_ref.as_ref();
    let mut res = String::with_capacity(char_str.len());
    let mut chars = char_str.chars();
    while let Some(c) = chars.next() {
        match c {
            '\\' => {
                let next = match chars.next() {
                    None => '/',
                    Some('0') => '\0',
                    Some('a') => '\u{07}',
                    Some('b') => '\u{08}',
                    Some('v') => '\u{0B}',
                    Some('f') => '\u{0C}',
                    Some('n') => '\n',
                    Some('r') => '\r',
                    Some('t') => '\t',
                    Some('e' | 'E') => '\u{1B}',
                    Some('\\') => '\\',
                    Some(c) => {
                        // otherwise don't consume the escape
                        res.push('\\');
                        c
                    }
                };
                res.push(next);
            }
            _ => res.push(c),
        };
    }
    res
}

/// Split a stream among several processes
///
/// xstream splits stdin by a given delimiter and pipes each section into a new process as the
/// stdin for that process. For very large streams of data, this is much more efficient than xargs.
/// The default usage spawns a new process for every section of stdin, even if the total number of
/// processes is limited. You can opt to reuse processes in a round-robin manner with the `--reuse`
/// option.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
#[clap(group(ArgGroup::new("delim").arg("null").conflicts_with("delimiter")))]
struct Cli {
    /// Set the delimiter between inputs
    ///
    /// Input sequences separated by this sequence will be sent to different processes.
    /// This will unescape common backslash escape sequences (0, a, b, v, f, n, r, t, e, and \).
    #[clap(short, long, value_parser = NonEmptyStringValueParser::new(), default_value = "\\n")]
    delimiter: String,

    /// Input streams are delimited by null characters
    ///
    /// This is equivalent to passing -d '\0'
    #[clap(short = '0', long)]
    null: bool,

    /// Set the delimiter to write at the end of child process inputs
    ///
    /// If specified, this delimiter will be written at the end of each sent input sequence,
    /// instead of the actual delimiter.
    /// Specify them empty string to remove the delimiter.
    /// This will unescape common backslash escape sequences (0, a, b, v, f, n, r, t, e, and \).
    #[clap(short, long)]
    write_delimiter: Option<String>,

    /// Run up to this many processes in parallel
    ///
    /// Specifying 0 will spawn unlimited processes
    #[clap(short, long, value_parser, default_value_t = 1)]
    parallel: usize,

    /// Reuse existing processes instead of spawning new ones
    #[clap(short, long)]
    reuse: bool,

    /// The command to execute for each delimited stream
    ///
    /// It is often helpful to prefix this with "--" so that other arguments are not interpreted by
    /// xstream
    #[clap(value_parser = NonEmptyStringValueParser::new())]
    command: String,

    /// Any additional arguments to command
    #[clap(value_parser)]
    args: Vec<String>,
}

fn main() {
    let args = Cli::parse();
    let delim = if args.null {
        "\0".to_owned()
    } else {
        unescape_delimiter(args.delimiter)
    };
    let write_delim = args.write_delimiter.map(unescape_delimiter);

    let mut command = Command::new(args.command);
    command.args(args.args);
    if args.reuse {
        let mut pool = Rotating::new(command, args.parallel);
        xstream_util::xstream(&mut pool, &mut io::stdin().lock(), &delim, &write_delim).unwrap();
    } else {
        let mut pool = Limiting::new(command, args.parallel);
        xstream_util::xstream(&mut pool, &mut io::stdin().lock(), &delim, &write_delim).unwrap();
    }
}

#[cfg(test)]
mod escape_tests {
    use super::unescape_delimiter;

    #[test]
    fn parse_null_escape() {
        assert_eq!(unescape_delimiter(r"\0"), "\0");
    }

    #[test]
    fn parse_newline_escape() {
        assert_eq!(unescape_delimiter(r"\n"), "\n");
    }

    #[test]
    fn parse_newline_raw() {
        assert_eq!(unescape_delimiter("\n"), "\n");
    }

    #[test]
    fn parse_extra_space() {
        assert_eq!(unescape_delimiter("\n "), "\n ");
    }
}

#[cfg(test)]
mod cli_tests {
    use super::Cli;
    use clap::CommandFactory;

    #[test]
    fn test_cli() {
        Cli::command().debug_assert()
    }
}
