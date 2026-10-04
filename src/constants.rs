pub static WIN_TABLE: [bool; 512] = {
    let mut table = [false; 512];

    #[rustfmt::skip]
    let win_masks = [
        0b000_000_111, 0b000_111_000, 0b111_000_000, // horizontal
        0b001_001_001, 0b010_010_010, 0b100_100_100, // vertical
        0b100_010_001, 0b001_010_100,                // diagonal
    ];

    let mut mask = 0;
    while mask < 512 {
        let mut i = 0;
        while i < 8 {
            if win_masks[i] & mask == win_masks[i] {
                table[mask] = true;
                break;
            }
            i += 1;
        }
        mask += 1;
    }
    table
};

static LOCAL_TO_GLOBAL_TABLE: [u8; 81] = {
    let mut table = [0; 81];

    let mut i = 0;
    while i < 81 {
        let main_cell = i / 9;
        let local_cell = i % 9;
        table[i as usize] =
            main_cell / 3 * 27 + main_cell % 3 * 3 + local_cell / 3 * 9 + local_cell % 3;
        i += 1;
    }
    table
};

pub const fn convert_global_coordinates(main_cell: u8, local_cell: u8) -> u8 {
    LOCAL_TO_GLOBAL_TABLE[(main_cell * 9 + local_cell) as usize]
}
