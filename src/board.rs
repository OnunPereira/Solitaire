use crate::get_lane_range;
use crate::CARD_HEIGHT;
use crate::CARD_WIDTH;
use crate::EMPTY;
use crate::PADDING;
use crate::PLAYFIELD_RANGE;
use crate::RANKS;
use crate::SUITS;
use crate::TOP_ROW_RANGE;
use futures::future::join_all;
use macroquad::prelude::mouse_position;
use macroquad::prelude::GREEN;
use macroquad::rand::ChooseRandom;
use macroquad::texture::Texture2D;
use macroquad::window::clear_background;

use crate::{card::Card, rank::Rank, suit::FrenchSuit};

type Lane = Vec<Card>;
type SuitStack = Vec<Card>;

pub struct HandMemory {
    pub card: Option<Card>,
    pub previous_stack: BoardArea,
}

pub enum BoardArea {
    Deck,
    Lane(usize),
    None,
    SuitStack(usize),
    Turned,
}

pub struct Board {
    pub deck: Vec<Card>,
    pub playfield: [Lane; 7],
    pub suit_stacks: [SuitStack; 4],
    pub turned: Vec<Card>,
    pub card_back_texture: Texture2D,
    pub hand_memory: HandMemory,
}

impl Board {
    pub fn add_card_to_lane(&mut self, mut card: Card, lane_number: usize) -> bool {
        let lane = &mut self.playfield[lane_number - 1];

        if let Some(last_card_in_lane) = lane.last() {
            let &Card { rank, suit, .. } = last_card_in_lane;

            let is_opposite_suit = match card.suit {
                FrenchSuit::Clubs | FrenchSuit::Spades => {
                    suit == FrenchSuit::Diamonds || suit == FrenchSuit::Hearts
                }
                FrenchSuit::Diamonds | FrenchSuit::Hearts => {
                    suit == FrenchSuit::Clubs || suit == FrenchSuit::Spades
                }
            };

            let can_stack = match card.rank {
                Rank::Ace => rank == Rank::Two,
                Rank::Two => rank == Rank::Three,
                Rank::Three => rank == Rank::Four,
                Rank::Four => rank == Rank::Five,
                Rank::Five => rank == Rank::Six,
                Rank::Six => rank == Rank::Seven,
                Rank::Seven => rank == Rank::Eight,
                Rank::Eight => rank == Rank::Nine,
                Rank::Nine => rank == Rank::Ten,
                Rank::Ten => rank == Rank::Jack,
                Rank::Jack => rank == Rank::Queen,
                Rank::Queen => rank == Rank::King,
                Rank::King => false,
            };

            if !is_opposite_suit || !can_stack {
                let _ = &self.return_card_to_previous_stack(card);
                return false;
            }

            card.update_pos(last_card_in_lane.x, last_card_in_lane.y + PADDING);
        }

        lane.push(card);

        true
    }

    pub fn add_card_to_suit(&mut self, mut card: Card, stack_number: usize) -> bool {
        let stack = &mut self.suit_stacks[stack_number - 1];

        if let Some(last_card_in_stack) = stack.last() {
            let &Card { rank, suit, .. } = last_card_in_stack;

            let can_stack = match rank {
                Rank::Ace => card.rank == Rank::Two,
                Rank::Two => card.rank == Rank::Three,
                Rank::Three => card.rank == Rank::Four,
                Rank::Four => card.rank == Rank::Five,
                Rank::Five => card.rank == Rank::Six,
                Rank::Six => card.rank == Rank::Seven,
                Rank::Seven => card.rank == Rank::Eight,
                Rank::Eight => card.rank == Rank::Nine,
                Rank::Nine => card.rank == Rank::Ten,
                Rank::Ten => card.rank == Rank::Jack,
                Rank::Jack => card.rank == Rank::Queen,
                Rank::Queen => card.rank == Rank::King,
                Rank::King => false,
            };

            if !can_stack || card.suit != suit {
                let _ = &self.return_card_to_previous_stack(card);
                return false;
            }

            card.update_pos(last_card_in_stack.x, last_card_in_stack.y);
        }

        stack.push(card);

        true
    }

    pub fn get_current_mouse_location() -> BoardArea {
        let (mx, my) = mouse_position();

        match (mx, my) {
            (x, y) if get_lane_range(1).contains(&x) => match y {
                y if TOP_ROW_RANGE.contains(&y) => BoardArea::SuitStack(1),
                y if PLAYFIELD_RANGE.contains(&y) => BoardArea::Lane(1),
                _ => BoardArea::None,
            },
            (x, y) if get_lane_range(2).contains(&x) => match y {
                y if TOP_ROW_RANGE.contains(&y) => BoardArea::SuitStack(2),
                y if PLAYFIELD_RANGE.contains(&y) => BoardArea::Lane(2),
                _ => BoardArea::None,
            },
            (x, y) if get_lane_range(3).contains(&x) => match y {
                y if TOP_ROW_RANGE.contains(&y) => BoardArea::SuitStack(3),
                y if PLAYFIELD_RANGE.contains(&y) => BoardArea::Lane(3),
                _ => BoardArea::None,
            },
            (x, y) if get_lane_range(4).contains(&x) => match y {
                y if TOP_ROW_RANGE.contains(&y) => BoardArea::SuitStack(4),
                y if PLAYFIELD_RANGE.contains(&y) => BoardArea::Lane(4),
                _ => BoardArea::None,
            },
            (x, y) if get_lane_range(5).contains(&x) => match y {
                y if PLAYFIELD_RANGE.contains(&y) => BoardArea::Lane(5),
                _ => BoardArea::None,
            },
            (x, y) if get_lane_range(6).contains(&x) => match y {
                y if TOP_ROW_RANGE.contains(&y) => BoardArea::Turned,
                y if PLAYFIELD_RANGE.contains(&y) => BoardArea::Lane(6),
                _ => BoardArea::None,
            },
            (x, y) if get_lane_range(7).contains(&x) => match y {
                y if TOP_ROW_RANGE.contains(&y) => BoardArea::Deck,
                y if PLAYFIELD_RANGE.contains(&y) => BoardArea::Lane(7),
                _ => BoardArea::None,
            },
            (_, _) => BoardArea::None,
        }
    }

    pub async fn initialize_deck(&mut self) {
        let mut futures = Vec::new();

        for rank in RANKS {
            for suit in SUITS {
                futures.push(Card::new(
                    get_lane_range(7).start,
                    TOP_ROW_RANGE.start,
                    rank,
                    suit,
                ));
            }
        }

        self.deck = join_all(futures).await;

        self.deck.shuffle();
    }

    pub fn initialize_playfield(&mut self) {
        (0..self.playfield.len()).for_each(|x| {
            (0..x + 1).for_each(|y| {
                let mut card = self.deck.pop().unwrap();

                card.update_pos(
                    PADDING + (x as f32) * (PADDING + CARD_WIDTH),
                    (y as f32) * PADDING + PADDING * 2.0 + CARD_HEIGHT,
                );

                if y == x {
                    card.toggle_visibility();
                }

                self.playfield[x].push(card);
            });
        });
    }

    pub fn new(card_back_texture: Texture2D) -> Self {
        Self {
            deck: EMPTY,
            playfield: [EMPTY; 7],
            suit_stacks: [EMPTY; 4],
            turned: EMPTY,
            card_back_texture,
            hand_memory: HandMemory {
                card: None,
                previous_stack: BoardArea::None,
            },
        }
    }

    pub fn paint(&mut self) {
        clear_background(GREEN);

        self.deck
            .iter()
            .for_each(|card| card.draw(&self.card_back_texture));

        self.turned
            .iter()
            .for_each(|card| card.draw(&self.card_back_texture));

        self.playfield.iter().for_each(|lane| {
            lane.iter()
                .for_each(|card| card.draw(&self.card_back_texture))
        });

        self.suit_stacks.iter().for_each(|lane| {
            lane.iter()
                .for_each(|card| card.draw(&self.card_back_texture))
        });

        if let Some(card) = &self.hand_memory.card {
            card.draw(&self.card_back_texture);
        }
    }

    pub fn return_card_to_previous_stack(&mut self, mut card: Card) -> bool {
        match self.hand_memory.previous_stack {
            BoardArea::Lane(x) => self.add_card_to_lane(card, x),
            BoardArea::SuitStack(x) => self.add_card_to_suit(card, x),
            BoardArea::Turned => {
                card.update_pos(get_lane_range(6).start, TOP_ROW_RANGE.start);
                self.turned.push(card);
                return true;
            }
            _ => false,
        };

        true
    }

    pub fn refill_deck(&mut self) {
        if self.deck.len() == 0 {
            self.deck.append(&mut self.turned);

            self.deck.iter_mut().for_each(|card| {
                card.is_hidden = false;
                card.is_turned = false;
                card.x = get_lane_range(7).start;
            });
        }
    }

    pub fn turn_card(&mut self) {
        if let Some(mut card) = self.deck.pop() {
            card.is_hidden = false;
            card.is_turned = true;
            card.x = get_lane_range(6).start;

            self.turned.push(card);
        }
    }
}
