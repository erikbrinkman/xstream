//! Command Line interface for xstream
//!
//! This just wraps the xstream lib in argument parsing, and character delimiter conversion

use std::convert::TryInto;
use std::io;
use std::process::Command;
use std::str;
use xstream_util;

use clap::{crate_version, App, Arg};

/// Escape delimiters in a string
///
/// The empty string becomes the null delimiter.
fn unescape_delimiter(char_str: &str) -> String {
    if char_str.is_empty() {
        String::from("\0")
    } else {
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
}

fn main() {
    let matches = App::new("xstream")
        .version(crate_version!())
        .author("Erik Brinkman <erik.brinkman@gmail.com>")
        .about("Split a stream among several processes")
        .arg(
            Arg::with_name("delim")
                .short("d")
                .long("delimiter")
                .help(
                    "Set the delimiter between inputs. This will unescape common backslash escape \
                    sequences (0, a, b, v, f, n, r, t, e, and \\). The empty string will be \
                    treated as the null delimiter.",
                )
                .default_value("\\n"),
        )
        .arg(
            Arg::with_name("null")
                .short("0")
                .long("null")
                .help("Input streams are delimited by null characters (\\0) instead of new lines.")
                .conflicts_with("delim"),
        )
        .arg(
            Arg::with_name("parallel")
                .short("p")
                .long("parallel")
                .help(
                    "run up to this many processes in parallel, specifying 0 will spawn unlimited \
                    processes",
                )
                .default_value("1"),
        )
        .arg(
            Arg::with_name("command")
                .required(true)
                .multiple(true)
                .help(
                    "The command to execute for each delimited stream. It is often helpful to \
                    prefix this with \"--\" so that other arguments are not interpreted by \
                    xstream.",
                ),
        )
        .after_help(
            "xstream splits stdin by a given delimiter and pipes each \
             section into a new process as the stdin for that \
             process. For very large streams of data, this is much \
             more efficient than xargs.",
        )
        .get_matches();

    // ----------------------------
    // Parse command line arguments
    // ----------------------------
    let mut command_iter = matches.values_of("command").unwrap(); // required
    let command = command_iter.next().unwrap(); // required
    let args: Vec<&str> = command_iter.collect();
    let delim_str = if matches.is_present("null") {
        ""
    } else {
        matches.value_of("delim").unwrap() // required
    };
    let delim = unescape_delimiter(&delim_str);
    let max_parallel: usize = matches
        .value_of("parallel")
        .unwrap() // required
        .parse::<i64>()
        .expect("couldn't parse parallel as integer")
        .try_into()
        .expect("parallel must be non-negative");

    // ---------
    // Main loop
    // ---------
    let stdin = io::stdin();
    let mut ihandle = stdin.lock();
    xstream_util::xstream(
        Command::new(command).args(args.iter()),
        &mut ihandle,
        delim.as_bytes(),
        max_parallel,
    )
    .unwrap();
}

#[cfg(test)]
mod tests {
    use super::unescape_delimiter;

    #[test]
    fn parse_empty() {
        assert_eq!(unescape_delimiter(r""), "\0");
    }

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
