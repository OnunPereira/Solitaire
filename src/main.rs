use std::ops::Range;

use futures::future::join_all;
use macroquad::{prelude::*, rand::ChooseRandom};

const CARD_WIDTH: f32 = 60.0;
const CARD_HEIGHT: f32 = 80.0;
const PADDING: f32 = 20.0;

const TOP_ROW_RANGE: Range<f32> = PADDING..PADDING + CARD_HEIGHT;
const PLAYFIELD_RANGE: Range<f32> = PADDING * 2. + CARD_HEIGHT..PADDING * 14. + CARD_HEIGHT * 2.;

struct HandMemory {
    card: Option<Card>,
    previous_stack: BoardArea,
}

enum BoardArea {
    Deck,
    Lane(usize),
    None,
    SuitStack(usize),
    Turned,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum FrenchSuit {
    Clubs,
    Diamonds,
    Hearts,
    Spades,
}

impl FrenchSuit {
    fn as_str(&self) -> &'static str {
        match self {
            FrenchSuit::Clubs => "clubs",
            FrenchSuit::Diamonds => "diamonds",
            FrenchSuit::Hearts => "hearts",
            FrenchSuit::Spades => "spades",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Rank {
    Ace,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,
    Queen,
    King,
}

impl Rank {
    fn as_str(&self) -> &'static str {
        match self {
            Rank::Two => "2",
            Rank::Three => "3",
            Rank::Four => "4",
            Rank::Five => "5",
            Rank::Six => "6",
            Rank::Seven => "7",
            Rank::Eight => "8",
            Rank::Nine => "9",
            Rank::Ten => "10",
            Rank::Jack => "jack",
            Rank::Queen => "queen",
            Rank::King => "king",
            Rank::Ace => "ace",
        }
    }
}

const SUITS: [FrenchSuit; 4] = [
    FrenchSuit::Clubs,
    FrenchSuit::Diamonds,
    FrenchSuit::Hearts,
    FrenchSuit::Spades,
];
const RANKS: [Rank; 13] = [
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

const EMPTY: Vec<Card> = Vec::new();

#[derive(Debug)]
struct Card {
    rank: Rank,
    suit: FrenchSuit,
    x: f32,
    y: f32,
    is_hidden: bool,
    is_turned: bool,
    texture: Texture2D,
}

// load_texture("res/card_back.png").await.unwrap();

impl Card {
    pub fn draw(&self, card_back_texture: &Texture2D) {
        if self.is_hidden {
            return;
        }

        let texture = match self.is_turned {
            true => self.texture,
            false => *card_back_texture,
        };

        draw_texture_ex(
            texture,
            self.x,
            self.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(CARD_WIDTH, CARD_HEIGHT)),
                source: None,
                rotation: 0.0,
                flip_x: false,
                flip_y: false,
                pivot: None,
            },
        );
    }

    pub async fn new(x: f32, y: f32, rank: Rank, suit: FrenchSuit) -> Self {
        println!("loading texture for {:?} of {:?} ", rank, suit);

        Self {
            rank,
            suit,
            x,
            y,
            is_hidden: false,
            is_turned: false,
            texture: load_texture(format!("{}_of_{}.png", rank.as_str(), suit.as_str()).as_str())
                .await
                .unwrap(),
        }
    }

    pub fn toggle_visibility(&mut self) {
        self.is_turned = !self.is_turned;
    }

    pub fn update_pos(&mut self, x: f32, y: f32) {
        self.x = x;
        self.y = y;
    }
}

type Lane = Vec<Card>;
type SuitStack = Vec<Card>;

struct Board {
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
                &self.return_card_to_previous_stack();
                return false;
            }

            card.update_pos(last_card_in_lane.x, last_card_in_lane.y + PADDING);
        }

        lane.push(card);
        true
    }

    pub fn add_card_to_suit(&mut self, card: Card, stack_number: usize) -> bool {
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
                &self.return_card_to_previous_stack();
                return false;
            }

            card.update_pos(last_card_in_stack.x, last_card_in_stack.y);
        }

        stack.push(card);
        true
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

    pub fn paint(&self) {
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

    pub fn return_card_to_previous_stack(&mut self) -> bool {
        if let Some(mut c) = self.hand_memory.card {
            match self.hand_memory.previous_stack {
                BoardArea::Lane(x) => self.add_card_to_lane(c, x),
                BoardArea::SuitStack(x) => self.add_card_to_suit(c, x),
                BoardArea::Turned => {
                    c.update_pos(get_lane_range(6).start, TOP_ROW_RANGE.start);
                    self.turned.push(c);
                    return true;
                }
                _ => false,
            };
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

fn get_distance_to_card_origin(card: &Card) -> Option<(f32, f32)> {
    let (mouse_x, mouse_y) = mouse_position();
    let Card { x, y, .. } = card;

    if (*x..*x + CARD_WIDTH).contains(&mouse_x) && (*y..*y + CARD_HEIGHT).contains(&mouse_y) {
        return Some((mouse_x - *x, mouse_y - *y));
    }

    None
}

fn get_lane_range(x: usize) -> Range<f32> {
    let final_edge = |y: usize| (CARD_WIDTH + PADDING) * (y as f32) + PADDING;

    final_edge(x - 1)..final_edge(x)
}

fn get_current_mouse_location() -> BoardArea {
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

#[macroquad::main("Solitaire")]
async fn main() {
    set_pc_assets_folder("assets");
    let card_back_texture = load_texture("card_back.png").await.unwrap();

    let mut board = Board::new(card_back_texture);

    board.initialize_deck().await;

    board.initialize_playfield();

    let mut distance_to_origin: (f32, f32) = (0., 0.);
    let mut has_moved_after_click: bool = false;

    loop {
        if is_mouse_button_pressed(MouseButton::Left) {
            match get_current_mouse_location() {
                BoardArea::Turned => {
                    if let Some(card) = board.turned.pop() {
                        if let Some(distance) = get_distance_to_card_origin(&card) {
                            distance_to_origin = distance;
                            board.hand_memory.card = Some(card);
                            board.hand_memory.previous_stack = BoardArea::Turned;
                        }
                    }
                }
                BoardArea::Deck => {
                    board.turn_card();
                }
                _ => (),
            }
        }

        if is_mouse_button_down(MouseButton::Left) {
            let (mouse_x, mouse_y) = mouse_position();
            let (dx, dy) = distance_to_origin;

            if let Some(card) = &mut board.hand_memory.card {
                card.update_pos(mouse_x - dx, mouse_y - dy);
            }

            if !has_moved_after_click {
                has_moved_after_click = true;
            }
        }

        if is_mouse_button_released(MouseButton::Left) {
            if !has_moved_after_click {
                match get_current_mouse_location() {
                    BoardArea::Turned => {
                        board.refill_deck();
                    }
                    _ => (),
                }
            } else if let Some(card) = board.hand_memory.card {
                board.hand_memory.card = None;

                match get_current_mouse_location() {
                    BoardArea::SuitStack(x) => {
                        board.add_card_to_suit(card, x);
                    }
                    BoardArea::Lane(x) => {
                        board.add_card_to_lane(card, x);
                    }
                    _ => {
                        board.return_card_to_previous_stack();
                    }
                }

                distance_to_origin = (0., 0.);
                has_moved_after_click = false;
                board.hand_memory.previous_stack = BoardArea::None;
            }
        }

        board.paint();

        next_frame().await
    }
}
