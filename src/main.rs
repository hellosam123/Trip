use crate::game::{MainBoard, Move};

mod constants;
mod game;

fn main() {
    let mut board = MainBoard::new();
    board.make_move(Move::new(4, 4));
    board.make_move(Move::new(4, 2));
    board.make_move(Move::new(4, 5));
    board.make_move(Move::new(4, 8));
    board.make_move(Move::new(4, 3));
    println!("{}", board);
}
