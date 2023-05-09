#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FrenchSuit {
    Clubs,
    Diamonds,
    Hearts,
    Spades,
}

impl FrenchSuit {
    pub fn as_str(&self) -> &'static str {
        match self {
            FrenchSuit::Clubs => "clubs",
            FrenchSuit::Diamonds => "diamonds",
            FrenchSuit::Hearts => "hearts",
            FrenchSuit::Spades => "spades",
        }
    }
}
