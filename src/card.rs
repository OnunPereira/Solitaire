use macroquad::prelude::load_texture;
use macroquad::{
    prelude::{vec2, WHITE},
    texture::{draw_texture_ex, DrawTextureParams, Texture2D},
};

use crate::{rank::Rank, suit::FrenchSuit, CARD_HEIGHT, CARD_WIDTH};

#[derive(Debug)]
pub struct Card {
    pub rank: Rank,
    pub suit: FrenchSuit,
    pub x: f32,
    pub y: f32,
    pub is_hidden: bool,
    pub is_turned: bool,
    texture: Texture2D,
}

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
