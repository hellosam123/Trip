use std::ops::{BitAnd, BitOr, Not};

use crate::constants::{self, convert_global_coordinates};

#[derive(Clone, Debug)]
pub struct MainBoard {
    pub main_board: Board,
    pub local_boards: [Board; 9],
    pub side_to_move: Player,
    pub board_to_move: Option<u8>,
}

// another alternative is to store both x and o together
#[derive(Clone, Copy, Debug)]
pub struct Board {
    pub x: Bitboard,
    pub o: Bitboard,
    pub state: BoardState,
}

// only requires u9, but uses u16 due to packing
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Bitboard(u16);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BoardState {
    InPlay,
    Drawn,
    Won(Player),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Player {
    X,
    O,
}

// xxxx_xxxx 8 bit structure, first 4 bits are main cell, last 4 bits are local cell
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Move(u8);

#[derive(Debug)]
pub struct MoveList {
    moves: [Move; 81],
    len: usize,
}

impl Move {
    pub const NULL: Self = Move(0b_1111_1111);
    const MAIN_MASK: u8 = 0b_1111_0000;
    const LOCAL_MASK: u8 = 0b_0000_1111;
    pub fn new(main_cell: u8, local_cell: u8) -> Self {
        Move(main_cell << 4 | local_cell)
    }

    pub fn get_main_cell(&self) -> u8 {
        (self.0 & Self::MAIN_MASK) >> 4
    }

    pub fn get_local_cell(&self) -> u8 {
        self.0 & Self::LOCAL_MASK
    }
}

impl MoveList {
    pub fn new() -> Self {
        Self {
            moves: [Move::NULL; 81],
            len: 0,
        }
    }

    pub fn push(&mut self, mv: Move) {
        self.moves[self.len] = mv;
        self.len += 1;
    }

    pub fn pop(&mut self) -> Move {
        self.len -= 1;
        self.moves[self.len]
    }

    pub fn clear(&mut self) {
        self.len = 0;
    }
}

impl Iterator for MoveList {
    type Item = Move;

    fn next(&mut self) -> Option<Self::Item> {
        if self.len == 0 {
            None
        } else {
            Some(self.pop())
        }
    }
}

impl Not for Player {
    type Output = Self;

    fn not(self) -> Self {
        match self {
            Self::X => Self::O,
            Self::O => Self::X,
        }
    }
}

impl Bitboard {
    pub const EMPTY: Self = Self(0);
    pub const BOARD_MASK: Self = Self(0b_111_111_111);

    pub fn set(&mut self, cell: u8) {
        self.0 |= 1 << cell;
    }

    pub fn contains(&self, cell: u8) -> bool {
        self.0 & (1 << cell) != 0
    }

    pub fn is_empty(self) -> bool {
        self & Self::BOARD_MASK == Self::EMPTY
    }

    pub fn is_full(self) -> bool {
        self & Self::BOARD_MASK == Self::BOARD_MASK
    }

    pub fn check_win(self) -> bool {
        constants::WIN_TABLE[self.0 as usize]
    }
}

impl BitOr for Bitboard {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Bitboard(self.0 | rhs.0)
    }
}

impl BitAnd for Bitboard {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Bitboard(self.0 & rhs.0)
    }
}

impl Not for Bitboard {
    type Output = Self;

    fn not(self) -> Self::Output {
        Bitboard(!self.0) & Self::BOARD_MASK
    }
}

impl Iterator for Bitboard {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        if self.is_empty() {
            None
        } else {
            let cell = self.0.trailing_zeros() as u8;
            self.0 &= self.0 - 1;
            Some(cell)
        }
    }
}

impl Default for Board {
    fn default() -> Self {
        Self {
            x: Bitboard::EMPTY,
            o: Bitboard::EMPTY,
            state: BoardState::InPlay,
        }
    }
}
impl Board {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn set(&mut self, cell: u8, player: Player) {
        match player {
            Player::X => self.x.set(cell),
            Player::O => self.o.set(cell),
        }
    }

    pub fn get_player(&self, cell: u8) -> Option<Player> {
        if self.x.contains(cell) {
            return Some(Player::X);
        }
        if self.o.contains(cell) {
            return Some(Player::O);
        }
        None
    }

    pub fn get_empty(&self) -> Bitboard {
        !(self.x | self.o)
    }

    pub fn is_full(&self) -> bool {
        (self.x | self.o).is_full()
    }

    pub fn check_state(&self) -> BoardState {
        if self.x.check_win() {
            return BoardState::Won(Player::X);
        }
        if self.o.check_win() {
            return BoardState::Won(Player::O);
        }
        if self.is_full() {
            return BoardState::Drawn;
        }
        BoardState::InPlay
    }

    pub fn set_state(&mut self) {
        self.state = self.check_state();
    }
}

impl Default for MainBoard {
    fn default() -> Self {
        Self {
            main_board: Board::new(),
            local_boards: [Board::new(); 9],
            side_to_move: Player::X,
            board_to_move: None,
        }
    }
}
impl MainBoard {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn generate_moves(&self) -> MoveList {
        let mut move_list = MoveList::new();
        if self.main_board.state != BoardState::InPlay {
            return move_list;
        }
        for main_cell in 0..9 {
            if let Some(board_to_move) = self.board_to_move
                && main_cell != board_to_move
            {
                continue;
            }
            let local_board = self.local_boards[main_cell as usize];
            if local_board.state != BoardState::InPlay {
                continue;
            }
            let empty_mask = local_board.get_empty();
            for local_cell in empty_mask {
                move_list.push(Move::new(main_cell, local_cell));
            }
        }

        move_list
    }

    pub fn make_move(&mut self, mv: Move) {
        let main_cell = mv.get_main_cell();
        let local_cell = mv.get_local_cell();
        self.local_boards[main_cell as usize].set(local_cell, self.side_to_move);
        self.local_boards[main_cell as usize].set_state();
        match self.local_boards[main_cell as usize].state {
            BoardState::Won(Player::X) => {
                self.main_board.set(main_cell, Player::X);
                self.main_board.set_state();
            }
            BoardState::Won(Player::O) => {
                self.main_board.set(main_cell, Player::O);
                self.main_board.set_state();
            }
            _ => (),
        }

        self.side_to_move = !self.side_to_move;
        if self.local_boards[local_cell as usize].state == BoardState::InPlay {
            self.board_to_move = Some(local_cell);
        } else {
            self.board_to_move = None;
        }
    }
}

impl std::fmt::Display for MainBoard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for row in 0..9 {
            for column in 0..9 {
                let main_cell = row / 3 * 3 + column / 3;
                let local_board = self.local_boards[main_cell as usize];
                let local_cell = row % 3 * 3 + column % 3;
                let content = local_board.get_player(local_cell);
                let display_str = match content {
                    Some(Player::X) => " X ",
                    Some(Player::O) => " O ",
                    None => "   ",
                };
                write!(f, "{}", display_str)?;
                if column % 3 == 2 && column < 8 {
                    write!(f, "│")?;
                } else if column < 8 {
                    write!(f, "┆")?;
                }
            }
            writeln!(f)?;

            if row % 3 == 2 && row < 8 {
                writeln!(f, "———————————│———————————│———————————")?;
            } else if row < 8 {
                writeln!(f, "---+---+---│---+---+---│---+---+---")?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn move_list() {
        let mut main_board = MainBoard::new();
        main_board.make_move(Move::new(4, 6));
        let move_list = main_board.generate_moves();
        println!("{:?}", move_list)
    }

    #[test]
    fn perft_startpos() {
        let main_board = MainBoard::new();
        for depth in 1..11 {
            let nodes = perft(&mut main_board.clone(), depth);
            println!("Depth {}: {}", depth, nodes);
        }
    }

    #[test]
    fn perft_verbose() {
        let main_board = MainBoard::new();
        let move_list = main_board.generate_moves();
        for mv in move_list {
            let mut move_board = main_board.clone();
            move_board.make_move(mv);
            let nodes = perft(&mut move_board, 2);
            println!("{:?}: {}", mv, nodes);
        }
    }

    fn perft(main_board: &mut MainBoard, depth: u8) -> u64 {
        if depth == 0 {
            1
        } else {
            let mut nodes = 0;
            let move_list = main_board.generate_moves();
            for mv in move_list {
                let mut move_board = main_board.clone();
                move_board.make_move(mv);
                nodes += perft(&mut move_board, depth - 1);
            }
            nodes
        }
    }
}
