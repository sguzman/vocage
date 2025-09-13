use std::io::{Error, ErrorKind};

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
    /// Construct from "arguments" (stringy flags).
    ///
    /// This is a **minimal, dependency-free** parser that recognizes a small set of flags:
    /// - `--columns A,B,C`
    /// - `--decks immediate,daily,weekly`
    /// - `--intervals 10,1440,2880`
    /// - `--showcolumns 0|1,2`   (side 0 shows [0]; side 1 shows [1,2])
    /// - `--listdelimiter ;`
    /// - `--header` / `--no-header`
    /// - `--returntofirst` / `--no-returntofirst`
    ///
    /// If you prefer `clap`, swap this body for your original parser; the signature stays the same.
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
                    columns = args[i + 1]
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect();
                    i += 2;
                }
                "--decks" if i + 1 < args.len() => {
                    decks = args[i + 1]
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect();
                    i += 2;
                }
                "--intervals" if i + 1 < args.len() => {
                    intervals = args[i + 1]
                        .split(',')
                        .filter(|s| !s.trim().is_empty())
                        .map(|s| s.trim().parse::<u32>())
                        .collect::<Result<Vec<_>, _>>()
                        .map_err(|e| Error::new(ErrorKind::InvalidInput, format!("invalid --intervals: {}", e)))?;
                    i += 2;
                }
                "--showcolumns" if i + 1 < args.len() => {
                    // Format: "0|1,2" => side0:[0], side1:[1,2]
                    let s = args[i + 1];
                    let parts: Vec<&str> = s.split('|').collect();
                    showcolumns.clear();
                    for side in parts {
                        let cols: Vec<u8> = side
                            .split(',')
                            .filter(|x| !x.trim().is_empty())
                            .map(|x| x.trim().parse::<u8>())
                            .collect::<Result<Vec<_>, _>>()
                            .map_err(|e| Error::new(ErrorKind::InvalidInput, format!("invalid --showcolumns: {}", e)))?;
                        showcolumns.push(cols);
                    }
                    if showcolumns.is_empty() {
                        showcolumns = vec![vec![0], vec![1]];
                    }
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
                    // ignore unknown switches; keep parity with "best-effort" behavior
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
}

