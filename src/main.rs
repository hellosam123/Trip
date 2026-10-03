use crate::game::{MainBoard, Move, MoveList};

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
    let mut move_list = MoveList::new();
    board.generate_moves(&mut move_list);
    println!("{:?}", move_list);
}
