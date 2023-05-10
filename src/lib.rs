use std::ops::Range;

use card::Card;
use macroquad::prelude::mouse_position;
use rank::Rank;
use suit::FrenchSuit;

pub mod board;
pub mod card;
pub mod rank;
pub mod suit;

pub const EMPTY: Vec<Card> = Vec::new();

pub const CARD_WIDTH: f32 = 60.0;
pub const CARD_HEIGHT: f32 = 80.0;
pub const PADDING: f32 = 20.0;

pub const TOP_ROW_RANGE: Range<f32> = PADDING..PADDING + CARD_HEIGHT;
pub const PLAYFIELD_RANGE: Range<f32> =
    (PADDING * 2. + CARD_HEIGHT)..(PADDING * 14. + CARD_HEIGHT * 2.);

pub const SUITS: [FrenchSuit; 4] = [
    FrenchSuit::Clubs,
    FrenchSuit::Diamonds,
    FrenchSuit::Hearts,
    FrenchSuit::Spades,
];
pub const RANKS: [Rank; 13] = [
    Rank::Ace,
    Rank::Two,
    Rank::Three,
    Rank::Four,
    Rank::Five,
    Rank::Six,
    Rank::Seven,
    Rank::Eight,
    Rank::Nine,
    Rank::Ten,
    Rank::Jack,
    Rank::Queen,
    Rank::King,
];

pub fn get_distance_to_card_origin(card: &Card) -> Option<(f32, f32)> {
    let (mouse_x, mouse_y) = mouse_position();
    let Card { x, y, .. } = card;

    if (*x..*x + CARD_WIDTH).contains(&mouse_x) && (*y..*y + CARD_HEIGHT).contains(&mouse_y) {
        return Some((mouse_x - *x, mouse_y - *y));
    }

    None
}

pub fn get_lane_range(x: usize) -> Range<f32> {
    let final_edge = |n: usize| (CARD_WIDTH + PADDING) * (n as f32) + PADDING;

    final_edge(x - 1)..final_edge(x)
}
