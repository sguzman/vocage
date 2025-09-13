use crate::model::card::VocaCard;
use crate::model::session::VocaSession;
use chrono::NaiveDateTime;
use rand::prelude::Rng;
use std::fs::File;
use std::io::{BufRead, BufReader, Error, ErrorKind, Write};
use std::path::Path;

/// A dataset = session config + cards + preserved comment lines
#[derive(Clone, Debug)]
pub struct VocaData {
    pub session: VocaSession,
    pub cards: Vec<VocaCard>,
    pub comments: Vec<(usize, String)>,
}

impl VocaData {
    /// Load a file into `VocaData`.
    ///
    /// **Assumptions:**
    /// - TSV file (tab-separated).
    /// - If `session.header == true`, the first non-comment line is header and sets `session.columns`
    ///   if empty; otherwise it is validated against existing columns.
    /// - Lines starting with `#` are preserved as comments.
    pub fn from_file(filename: &str, reset: bool) -> Result<Self, Error> {
        let file = File::open(filename)?;
        let mut reader = BufReader::new(file);

        let mut session = VocaSession {
            columns: Vec::new(),
            decks: vec![
                "immediate".into(),
                "daily".into(),
                "weekly".into(),
                "monthly".into(),
            ],
            intervals: vec![0, 60 * 24, 60 * 24 * 7, 60 * 24 * 30], // default fallbacks
            returntofirst: true,
            showcolumns: vec![vec![0], vec![1]],
            listdelimiter: None,
            header: true,
            filename: Some(filename.to_string()),
        };

        let mut cards: Vec<VocaCard> = Vec::new();
        let mut comments: Vec<(usize, String)> = Vec::new();

        let mut line_nr = 0usize;
        let mut buf = String::new();

        // read entire file
        while {
            buf.clear();
            reader.read_line(&mut buf)? > 0
        } {
            line_nr += 1;
            let line = buf.trim_end_matches(&['\r', '\n'][..]).to_string();

            if line.trim().is_empty() {
                continue;
            }
            if line.starts_with('#') {
                comments.push((line_nr, line));
                continue;
            }

            // header
            if session.header && session.columns.is_empty() {
                let cols: Vec<String> = line.split('\t').map(|s| s.trim().to_string()).collect();
                if cols.is_empty() {
                    return Err(Error::new(
                        ErrorKind::InvalidData,
                        format!("empty header at {}", filename),
                    ));
                }
                session.columns = cols;
                continue;
            }

            // data row
            let card = VocaCard::parse_line(&line, reset, line_nr)?;
            cards.push(card);
        }

        Ok(Self {
            session,
            cards,
            comments,
        })
    }

    /// Random presentable card index within optional deck subset.
    pub fn random_index(
        &self,
        rng: &mut impl Rng,
        decks: Option<&Vec<u8>>,
        due_only: bool,
        seen_only: bool,
    ) -> Option<(usize, usize)> {
        let now = chrono::Utc::now().naive_utc();
        let filtered: Vec<usize> = self
            .cards
            .iter()
            .enumerate()
            .filter(|(_, c)| c.is_presentable(Some(&now), decks, due_only, seen_only))
            .map(|(i, _)| i)
            .collect();

        if filtered.is_empty() {
            None
        } else {
            let pick = rng.gen_range(0..filtered.len());
            Some((filtered[pick], filtered.len()))
        }
    }

    /// Next presentable card index after `index`.
    /// If `inclusive == true`, the current one may be returned if presentable.
    pub fn next_index(
        &self,
        index: usize,
        decks: Option<&Vec<u8>>,
        due_only: bool,
        seen_only: bool,
        inclusive: bool,
    ) -> Option<(usize, usize)> {
        let now = chrono::Utc::now().naive_utc();

        let mut count = 0usize;
        let n = self.cards.len();
        if n == 0 {
            return None;
        }

        let mut i = if inclusive { index } else { (index + 1) % n };
        loop {
            if self.cards[i].is_presentable(Some(&now), decks, due_only, seen_only) {
                return Some((i, count + 1));
            }
            i = (i + 1) % n;
            count += 1;
            if (!inclusive && i == (index + 1) % n) || (inclusive && i == index) {
                break;
            }
        }
        None
    }

    /// Save dataset back to its source file.
    pub fn write(&self, _reset: bool) -> Result<(), std::io::Error> {
        let filename = self
            .session
            .filename
            .as_deref()
            .ok_or_else(|| Error::new(ErrorKind::Other, "no filename set on session"))?;

        let mut f = File::create(filename)?;
        // write preserved comments at the top
        for (_line_nr, comment) in &self.comments {
            writeln!(f, "{}", comment)?;
        }
        // header
        if self.session.header && !self.session.columns.is_empty() {
            writeln!(f, "{}", self.session.columns.join("\t"))?;
        }
        // rows
        let width = self.session.columns.len().max(1);
        for card in &self.cards {
            let s = card.write_to_string(width, _reset);
            writeln!(f, "{}", s)?;
        }
        Ok(())
    }
}
