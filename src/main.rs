use crate::game::{MainBoard, Move};

mod constants;
mod game;
mod net;

fn main() {
    let mut board = MainBoard::new();
    println!("{}", board);

    loop {
        let mut input = String::new();

        std::io::stdin().read_line(&mut input).unwrap();

        let trimmed = input.trim();

        let mut iter = trimmed.split_whitespace().map(|s| s.parse::<u8>());

        let main_cell = iter.next().unwrap().unwrap() - 1;
        let local_cell = iter.next().unwrap().unwrap() - 1;

        let mv = Move::new(main_cell, local_cell);
        let legal_moves = board.generate_moves();
        let mut legal = false;
        for legal_move in legal_moves {
            if mv == legal_move {
                legal = true;
                break;
            }
        }

        if legal == true {
            board.make_move(Move::new(main_cell, local_cell));
        } else {
            println!("Enter a valid move!");
        }
        println!("{}", board);
    }
}
