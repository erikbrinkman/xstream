use std::io::{self, BufRead, Write};
use std::process::{Command, Stdio};

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
        .version("1.0")
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
    let mut command_iter = matches.values_of("command").unwrap();
    let command = command_iter.next().unwrap(); // required arg
    let args: Vec<&str> = command_iter.collect();
    let delim_str = if matches.is_present("null") {
        ""
    } else {
        matches.value_of("delim").unwrap() // arg with default value
    };
    let delim = parse_character(&delim_str).expect("invalid delimiter") as u8;

    // ---------
    // Main loop
    // ---------
    let stdin = io::stdin();
    let mut ihandle = stdin.lock();

    while {
        // Covers each command invocation
        let mut proc = Command::new(command)
            .args(args.iter())
            .stdin(Stdio::piped())
            .stdout(Stdio::inherit())
            .spawn()
            .expect("failed to start child process");
        let mut hit_delim;
        {
            let ohandle = proc
                .stdin
                .as_mut()
                .expect("failed to capture child process stdin");
            while {
                let size = {
                    let buf = ihandle.fill_buf().expect("failed to read from stdin");
                    let mut itr = buf.splitn(2, |&c| c == delim);
                    let dump = itr.next().unwrap();
                    hit_delim = itr.next().is_some();
                    ohandle
                        .write_all(dump)
                        .expect("failed to pipe data to child process");
                    dump.len()
                };
                ihandle.consume(size + (hit_delim as usize));
                !hit_delim && size > 0
            } {}
        }

        match proc.wait().expect("child process never started").code() {
            Some(0) => (),
            Some(_) => panic!("child process finished with nonzero exit code"),
            None => panic!("child process was killed by a signal"),
        };

        hit_delim
    } {}
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
