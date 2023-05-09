use macroquad::prelude::*;
use solitaire::{
    board::{Board, BoardArea},
    get_distance_to_card_origin,
};

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
            match Board::get_current_mouse_location() {
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
                match Board::get_current_mouse_location() {
                    BoardArea::Turned => {
                        board.refill_deck();
                    }
                    _ => (),
                }
            } else if let Some(card) = board.hand_memory.card {
                board.hand_memory.card = None;

                match Board::get_current_mouse_location() {
                    BoardArea::SuitStack(x) => {
                        board.add_card_to_suit(card, x);
                    }
                    BoardArea::Lane(x) => {
                        board.add_card_to_lane(card, x);
                    }
                    _ => {
                        board.return_card_to_previous_stack(card);
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
