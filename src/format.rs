/// Controls how a card is printed.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PrintFormat {
    Plain,
    AnsiColour,
}
