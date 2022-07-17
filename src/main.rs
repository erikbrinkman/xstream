//! Command Line interface for xstream
//!
//! This just wraps the xstream lib in argument parsing, and character delimiter conversion

use clap::builder::NonEmptyStringValueParser;
use clap::{ArgGroup, Parser};
use std::io;
use std::process::Command;
use std::str;

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
                    Some('e') | Some('E') => '\u{1B}',
                    Some('\\') => '\\',
                    Some(c) => {
                        // otherwise don't consume the escape
                        res.push('\\');
                        c
                    }
                };
                res.push(next)
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
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
#[clap(group(ArgGroup::new("delim").arg("null").conflicts_with("delimiter")))]
struct Args {
    /// Set the delimiter between inputs
    ///
    /// This will unescape common backslash escape sequences (0, a, b, v, f, n, r, t, e, and \).
    /// The empty string will be treated as the null delimiter.
    #[clap(short, long, value_parser = NonEmptyStringValueParser::new(), default_value = "\\n")]
    delimiter: String,

    /// Input streams are delimited by null characters
    ///
    /// This is equivalent to passing -d '\0'
    #[clap(short = '0', long)]
    null: bool,

    /// Run up to this many processes in parallel
    ///
    /// Specifying 0 will spawn unlimited processes
    #[clap(short, long, value_parser, default_value_t = 1)]
    parallel: usize,

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
    let args = Args::parse();
    let delim = if args.null {
        "\0".to_owned()
    } else {
        unescape_delimiter(args.delimiter)
    };

    let stdin = io::stdin();
    let mut ihandle = stdin.lock();
    xstream_util::xstream(
        Command::new(args.command).args(args.args.iter()),
        &mut ihandle,
        delim.as_bytes(),
        args.parallel,
    )
    .unwrap();
}

#[cfg(test)]
mod tests {
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
