use std::collections::VecDeque;

pub mod bot;
pub mod gen_moves;
pub mod field;
pub mod state;
pub mod mov;
pub mod evaluator;

pub use field::Field;
pub use state::State;
pub use mov::Move;
pub use gen_moves::gen_moves;

#[derive(PartialEq, Eq, Hash, Copy, Clone, Debug)]
pub enum Piece {
    J = 0,
    L = 1,
    S = 2,
    Z = 3,
    T = 4,
    I = 5,
    O = 6,
    None,
}
pub const PIECE_MAP: [[u32; 4]; 7] = [
    [ // J
        0b100_111_000,
        0b011_010_010,
        0b000_111_001,
        0b010_010_110
    ], 
    [ // L
        0b001_111_000,
        0b110_010_010,
        0b000_111_100,
        0b010_010_011
    ], 
    [ // S
        0b011_110_000,
        0b010_011_001,
        0b000_011_110,
        0b100_110_010
    ], 
    [ // Z
        0b110_011_000,
        0b001_011_010,
        0b000_110_011,
        0b010_110_100
    ], 
    [ // T
        0b010_111_000,
        0b010_011_010,
        0b000_111_010,
        0b010_110_010
    ], 
    [ // I
        0b00000_00000_01111_00000_00000,
        0b00000_00010_00010_00010_00010,
        0b00000_00000_00000_01111_00000,
        0b00000_00100_00100_00100_00100,
    ],
    [ // O
        0b011_011_000,
        0b000_011_011,
        0b000_110_110,
        0b110_110_000
    ], 
];

#[derive(Copy, Clone)]
pub enum Clear {
    None,
    Clear1,
    Clear2,
    Clear3,
    Clear4,
    Tspin1,
    Tspin2,
    Tspin3,
}

#[derive(PartialEq)]
pub enum Key {
    Left,
    Right,
    Cw,
    Ccw,
    _180,
    HardDrop,
    SoftDrop,
    Hold,
}

#[derive(Copy, Clone)]
pub struct Props {
    pub sum_atk: u8,
    pub sum_ds: u8,
    pub b2b: u8,
    pub combo: u8,
    pub clear: Clear
}
impl Props {
    pub fn new () -> Self {
        Self {
            b2b: 0,
            combo: 0,
            sum_atk: 0,
            sum_ds: 0,
            clear: Clear::None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn field_check_conflict_test () {
        let field: Field = Field::new();
        let mut mov: Move = Move::new();
        let p: Piece = Piece::L;

        mov.y = 19;
        mov.x = 0;
        assert_eq!(field.check_conflict(&mov, &p), true);
    }
    
    #[test]
    fn move_drop_test () {
        let field: Field = Field::new();
        let mut mov: Move = Move::new();
        let p: Piece = Piece::L;

        mov.apply_key(&Key::HardDrop, &field, &p, &p);

        assert_eq!(mov.x, 4);
        assert_eq!(mov.y, 19);
        assert_eq!(mov.r, 0);
    }

    #[test] 
    fn move_apply_key_test () {
        let field: Field = Field::new();
        let mut mov: Move = Move::new();
        let p: Piece = Piece::L;
        let h: Piece = Piece::J;

        mov.apply_key(&Key::Cw, &field, &p, &h);
        mov.apply_key(&Key::Left, &field, &p, &h);
        mov.apply_key(&Key::Left, &field, &p, &h);
        mov.apply_key(&Key::Left, &field, &p, &h);
        //mov.apply_key(&Key::Left, &field, &p, &h);

        mov.apply_key(&Key::HardDrop, &field, &p, &h);

        assert_eq!(mov.x, 1);
        assert_eq!(mov.y, 18);
        assert_eq!(mov.r, 1);
    }

    #[test] 
    fn field_apply_move_test () {
        let mut field: Field = Field::new();
        let mut mov: Move = Move::new();
        let p: Piece = Piece::L;
        let h: Piece = Piece::J;

        //mov.apply_key(&Key::Hold, &field, &p, &h);
        //mov.apply_key(&Key::Cw, &field, &p, &h);
        //mov.apply_key(&Key::Left, &field, &p, &h);
        //mov.apply_key(&Key::Left, &field, &p, &h);
        //mov.apply_key(&Key::Left, &field, &p, &h);
        //mov.apply_key(&Key::Left, &field, &p, &h);

        mov.apply_key(&Key::HardDrop, &field, &p, &h);

        field = field.apply_move(&mov, &p, &h);

        assert_eq!(field.m[17], 0b00000_00000);
        assert_eq!(field.m[18], 0b00001_00000);
        assert_eq!(field.m[19], 0b00001_11000);
    }
}
