//! Command Line interface for xstream
//!
//! This just wraps the xstream lib in argument parsing, and character delimiter conversion

use std::convert::TryInto;
use std::io;
use std::process::Command;
use xstream_util;

use clap::{App, Arg};

/// Parse a string as a single character
fn parse_character(char_str: &str) -> Result<char, &str> {
    match char_str {
        "" | "\\0" => Ok('\0'),
        "\\t" => Ok('\t'),
        "\\r" => Ok('\r'),
        "\\n" => Ok('\n'),
        "\\\\" => Ok('\\'),
        string if string.len() == 1 => Ok(string.as_bytes()[0] as char),
        _ => Err("could not interpret string as a character"),
    }
}

fn main() {
    let matches = App::new("xstream")
        .version("1.1")
        .author("Erik Brinkman <erik.brinkman@gmail.com>")
        .about("Split a stream among several processes")
        .arg(
            Arg::with_name("delim")
                .short("d")
                .long("delimiter")
                .help(
                    "Set the delimiter between inputs. \
                     This also accepts {\\0, \\t, \\r, \\n, and \\\\}.",
                )
                .default_value("\\n"),
        )
        .arg(
            Arg::with_name("null")
                .short("0")
                .long("null")
                .help(
                    "Input streams are delimited by null characters (\\0) \
                     instead of new lines.",
                )
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
                    "The command to execute for each delimited stream. \
                     It is often helpful to prefix this with \"--\" so that \
                     other arguments are not interpreted by xstream.",
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
    let mut command_iter = matches.values_of("command").expect("command is required");
    let command = command_iter.next().expect("command is required");
    let args: Vec<&str> = command_iter.collect();
    let delim_str = if matches.is_present("null") {
        ""
    } else {
        matches.value_of("delim").expect("delim has default value")
    };
    let delim = parse_character(&delim_str).expect("invalid delimiter") as u8;
    let max_parallel: usize = matches
        .value_of("parallel")
        .expect("parallel has default value")
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
        delim,
        max_parallel,
    )
    .unwrap();
}

#[cfg(test)]
mod tests {
    use super::parse_character;

    #[test]
    fn parse_empty() {
        assert_eq!(parse_character(""), Ok('\0'));
    }

    #[test]
    fn parse_null_escape() {
        assert_eq!(parse_character("\\0"), Ok('\0'));
    }

    #[test]
    fn parse_newline_escape() {
        assert_eq!(parse_character("\\n"), Ok('\n'));
    }

    #[test]
    fn parse_newline_raw() {
        assert_eq!(parse_character("\n"), Ok('\n'));
    }

    #[test]
    fn parse_extra() {
        assert!(parse_character("\n ").is_err());
    }
}
