use crate::format::PrintFormat;
use crate::model::session::VocaSession;
use ansi_term::Colour;
use chrono::{Duration, NaiveDateTime, Utc};
use std::fmt;

/// A single flashcard
#[derive(Clone, Debug)]
pub struct VocaCard {
    pub fields: Vec<String>,
    pub due: Option<NaiveDateTime>,
    /// index into `VocaSession.decks`
    pub deck: u8,
}

impl VocaCard {
    /// Parse a data line from a TSV-like file into a card.
    ///
    /// **Assumption:** We split on '\t' and treat the entire row as fields.
    /// `reset == true` clears due/deck information.
    /// If the file encodes due/deck in dedicated trailing columns in your original format,
    /// adapt here (or call a custom parser).
    pub fn parse_line(line: &str, reset: bool, _linenr: usize) -> Result<VocaCard, std::io::Error> {
        let mut fields: Vec<String> = line.split('\t').map(|s| s.trim_end().to_string()).collect();
        if fields.is_empty() {
            fields.push(String::new());
        }

        let mut card = VocaCard {
            fields,
            due: None,
            deck: 0,
        };

        if reset {
            card.due = None;
            card.deck = 0;
        }

        Ok(card)
    }

    /// Serialize a card back to a line string.
    /// `columncount` pads/truncates field count to a consistent width.
    pub fn write_to_string(&self, columncount: usize, _reset: bool) -> String {
        let mut cols = self.fields.clone();
        if cols.len() < columncount {
            cols.resize(columncount, String::new());
        } else if cols.len() > columncount {
            cols.truncate(columncount);
        }
        cols.join("\t")
    }

    /// Move card to an explicit deck; returns `true` if deck changed.
    pub fn move_to_deck(&mut self, deck: u8, session: &VocaSession) -> bool {
        let changed = self.deck != deck;
        self.deck = deck;

        // Update due using the per-deck intervals if available
        if let Some(minutes) = session
            .intervals
            .get(self.deck as usize)
            .copied()
            .map(|m| m as i64)
        {
            let now = Utc::now().naive_utc();
            self.due = Some(now + Duration::minutes(minutes));
        }

        changed
    }

    /// Promote card by one deck (capped at last).
    pub fn promote(&mut self, session: &VocaSession) -> bool {
        let next = (self.deck as usize + 1).min(session.decks.len().saturating_sub(1)) as u8;
        self.move_to_deck(next, session)
    }

    /// Demote card by one deck (or to first if configured).
    pub fn demote(&mut self, session: &VocaSession) -> bool {
        let target = if session.returntofirst {
            0
        } else {
            self.deck.saturating_sub(1)
        };
        self.move_to_deck(target, session)
    }

    /// Print the card to stdout using the given format and side.
    pub fn print(
        &self,
        side: u8,
        session: &VocaSession,
        format: PrintFormat,
        wraplist: bool,
    ) -> Result<(), fmt::Error> {
        let rows = self.fields_to_str(side, session, wraplist)?;
        for (col, text) in rows {
            match format {
                PrintFormat::Plain => println!("{text}"),
                PrintFormat::AnsiColour => {
                    println!("{}", Colour::Green.paint(format!("[{}] {text}", col)))
                }
            }
        }
        Ok(())
    }

    /// Render the configured columns for a side into displayable rows.
    /// Returns (column_index, text) for each displayed item.
    pub fn fields_to_str(
        &self,
        side: u8,
        session: &VocaSession,
        wraplist: bool,
    ) -> Result<Vec<(u8, &str)>, fmt::Error> {
        let mut out: Vec<(u8, &str)> = Vec::new();
        let cols = session
            .showcolumns
            .get(side as usize)
            .cloned()
            .unwrap_or_default();
        for &idx in &cols {
            for piece in self.field_to_str(idx, session, wraplist)? {
                out.push((idx, piece));
            }
        }
        Ok(out)
    }

    /// Slice a single field into lines, possibly wrapping a list delimiter.
    pub fn field_to_str(
        &self,
        index: u8,
        session: &VocaSession,
        wraplist: bool,
    ) -> Result<Vec<&str>, fmt::Error> {
        let idx = index as usize;
        if idx >= self.fields.len() {
            return Ok(vec![]);
        }
        let field = self.fields[idx].as_str();
        if wraplist {
            if let Some(delim) = &session.listdelimiter {
                if !delim.is_empty() {
                    let parts: Vec<&str> = field
                        .split(delim)
                        .map(|s| s.trim())
                        .filter(|s| !s.is_empty())
                        .collect();
                    return Ok(parts);
                }
            }
        }
        Ok(vec![field])
    }

    /// Whether the card should be shown given decks/due filters.
    pub fn is_presentable(
        &self,
        now: Option<&NaiveDateTime>,
        decks: Option<&Vec<u8>>,
        due_only: bool,
        seen_only: bool,
    ) -> bool {
        if let Some(ds) = decks {
            if !ds.contains(&self.deck) {
                return false;
            }
        }

        if seen_only && self.due.is_none() {
            return false;
        }

        if due_only {
            let now = *now.unwrap_or(&Utc::now().naive_utc());
            match self.due {
                Some(due) => due <= now,
                None => false,
            }
        } else {
            true
        }
    }
}
