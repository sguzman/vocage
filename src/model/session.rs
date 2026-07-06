use std::io::{Error, ErrorKind};

// add clap v2
use clap::{Arg, ArgMatches};

/// Session-wide configuration/state
#[derive(Clone, Debug)]
pub struct VocaSession {
    pub columns: Vec<String>,
    pub decks: Vec<String>,
    /// interval in minutes per deck index
    pub intervals: Vec<u32>,
    pub returntofirst: bool,
    /// Configuration of columns to show for each side of the card (e.g., vec![vec![0], vec![1,2]])
    pub showcolumns: Vec<Vec<u8>>,
    /// list delimiter to wrap multi-values within a single field (e.g., ";")
    pub listdelimiter: Option<String>,
    /// if `true`, the input has a header row with column names
    pub header: bool,
    /// not exported in original; keep private but useful for `write`
    pub(crate) filename: Option<String>,
}

impl VocaSession {
    /// Construct from "arguments" (stringy flags) without clap.
    /// You can keep using this for programmatic calls or tests.
    pub fn from_arguments(args: Vec<&str>) -> Result<Self, Error> {
        let mut columns: Vec<String> = Vec::new();
        let mut decks: Vec<String> = Vec::new();
        let mut intervals: Vec<u32> = Vec::new();
        let mut showcolumns: Vec<Vec<u8>> = vec![vec![0], vec![1]];
        let mut listdelimiter: Option<String> = None;
        let mut header = true;
        let mut returntofirst = true;

        let mut i = 0;
        while i < args.len() {
            match args[i] {
                "--columns" if i + 1 < args.len() => {
                    columns = parse_csv_strings(args[i + 1]);
                    i += 2;
                }
                "--decks" if i + 1 < args.len() => {
                    decks = parse_csv_strings(args[i + 1]);
                    i += 2;
                }
                "--intervals" if i + 1 < args.len() => {
                    intervals = parse_csv_u32(args[i + 1])?;
                    i += 2;
                }
                "--showcolumns" if i + 1 < args.len() => {
                    showcolumns = parse_showcolumns(args[i + 1])?;
                    i += 2;
                }
                "--listdelimiter" if i + 1 < args.len() => {
                    listdelimiter = Some(args[i + 1].to_string());
                    i += 2;
                }
                "--header" => {
                    header = true;
                    i += 1;
                }
                "--no-header" => {
                    header = false;
                    i += 1;
                }
                "--returntofirst" => {
                    returntofirst = true;
                    i += 1;
                }
                "--no-returntofirst" => {
                    returntofirst = false;
                    i += 1;
                }
                _ => {
                    // ignore unknown switches
                    i += 1;
                }
            }
        }

        if !intervals.is_empty() && !decks.is_empty() && intervals.len() != decks.len() {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                format!(
                    "intervals length ({}) must match decks length ({})",
                    intervals.len(),
                    decks.len()
                ),
            ));
        }

        Ok(Self {
            columns,
            decks,
            intervals,
            returntofirst,
            filename: None,
            showcolumns,
            listdelimiter,
            header,
        })
    }

    /// Return deck index by name, if present.
    pub fn get_deck_by_name(&self, name: &str) -> Option<u8> {
        self.decks.iter().position(|d| d == name).map(|i| i as u8)
    }

    // ===== clap v2 interop expected by src/bin/vocage.rs =====

    /// Arguments that the binary attaches to its `App`.
    /// (clap v2 types)
    pub fn common_arguments() -> Vec<Arg<'static, 'static>> {
        vec![
            Arg::with_name("columns")
                .long("columns")
                .value_name("COL1,COL2,...")
                .takes_value(true)
                .help("Comma-separated column names (header row overrides if present)"),
            Arg::with_name("decks")
                .long("decks")
                .value_name("D1,D2,...")
                .takes_value(true)
                .help("Comma-separated deck names, e.g. immediate,daily,weekly"),
            Arg::with_name("intervals")
                .long("intervals")
                .value_name("MINS,...")
                .takes_value(true)
                .help("Comma-separated deck intervals in minutes; must match --decks length"),
            Arg::with_name("showcolumns")
                .long("showcolumns")
                .value_name("SPEC")
                .takes_value(true)
                .help(r#"Column spec per side, e.g. "0|1,2" (side0 shows [0]; side1 shows [1,2])"#),
            Arg::with_name("listdelimiter")
                .long("listdelimiter")
                .value_name("STR")
                .takes_value(true)
                .help("If set, fields containing this delimiter are printed as multiple lines"),
            Arg::with_name("header")
                .long("header")
                .help("Input has a header row with column names")
                .takes_value(false),
            Arg::with_name("no-header")
                .long("no-header")
                .help("Input has no header row")
                .takes_value(false),
            Arg::with_name("returntofirst")
                .long("returntofirst")
                .help("Demotion returns a card to the first deck")
                .takes_value(false),
            Arg::with_name("no-returntofirst")
                .long("no-returntofirst")
                .help("Demotion only steps down one deck")
                .takes_value(false),
        ]
    }

    /// Apply parsed matches to this session (keeps unspecified values as-is).
    pub fn set_common_arguments(&mut self, m: &ArgMatches) -> Result<(), Error> {
        if let Some(s) = m.value_of("columns") {
            self.columns = parse_csv_strings(s);
        }
        if let Some(s) = m.value_of("decks") {
            self.decks = parse_csv_strings(s);
        }
        if let Some(s) = m.value_of("intervals") {
            self.intervals = parse_csv_u32(s)?;
        }
        if let Some(s) = m.value_of("showcolumns") {
            self.showcolumns = parse_showcolumns(s)?;
        }
        if let Some(s) = m.value_of("listdelimiter") {
            self.listdelimiter = Some(s.to_string());
        }

        // booleans via paired flags
        if m.is_present("no-header") {
            self.header = false;
        } else if m.is_present("header") {
            self.header = true;
        }

        if m.is_present("no-returntofirst") {
            self.returntofirst = false;
        } else if m.is_present("returntofirst") {
            self.returntofirst = true;
        }

        if !self.intervals.is_empty()
            && !self.decks.is_empty()
            && self.intervals.len() != self.decks.len()
        {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                format!(
                    "intervals length ({}) must match decks length ({})",
                    self.intervals.len(),
                    self.decks.len()
                ),
            ));
        }

        Ok(())
    }
}

// ---------- helpers ----------

fn parse_csv_strings(s: &str) -> Vec<String> {
    s.split(',')
        .map(|x| x.trim().to_string())
        .filter(|x| !x.is_empty())
        .collect()
}

fn parse_csv_u32(s: &str) -> Result<Vec<u32>, Error> {
    s.split(',')
        .map(|x| x.trim())
        .filter(|x| !x.is_empty())
        .map(|x| {
            x.parse::<u32>().map_err(|e| {
                Error::new(
                    ErrorKind::InvalidInput,
                    format!("invalid integer '{}': {}", x, e),
                )
            })
        })
        .collect()
}

fn parse_showcolumns(s: &str) -> Result<Vec<Vec<u8>>, Error> {
    // "0|1,2" => vec![vec![0], vec![1,2]]
    let mut out: Vec<Vec<u8>> = Vec::new();
    for side in s.split('|') {
        let vecu8: Vec<u8> = side
            .split(',')
            .map(|x| x.trim())
            .filter(|x| !x.is_empty())
            .map(|x| {
                x.parse::<u8>().map_err(|e| {
                    Error::new(
                        ErrorKind::InvalidInput,
                        format!("invalid column '{}': {}", x, e),
                    )
                })
            })
            .collect::<Result<_, _>>()?;
        out.push(vecu8);
    }
    if out.is_empty() {
        Ok(vec![vec![0], vec![1]])
    } else {
        Ok(out)
    }
}
