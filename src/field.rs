use super::{Piece, Move, Props};
use super::mac::*;

use std::fmt;

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct Field {
    pub m: [u16; 20],
}
impl fmt::Display for Field {
    fn fmt (&self, f: &mut fmt::Formatter) -> fmt::Result { 
        for y in 0..20 {
            for x in 0..10 {
                let b: bool = (self.m[y] & (1 << x)) >> x == 1;
                if b {
                    write!(f, "# ")?;
                } else {
                    write!(f, ". ")?;
                }
            }
            write!(f, "\n")?;
        }
        Ok(())
    }
}

impl Field {
    pub fn new () -> Self {
        Self {
            m: [0; 20],
        }
    }
    pub fn check_conflict (self: &Self, m: &Move, p: &Piece) -> bool {
        let map: &u32 = &PIECE_MAP[*p as usize][m.r as usize];
        let n: i8 = if *p == Piece::I {5} else {3};
        let c_x: i8 = m.x - n/2;
        let c_y: i8 = m.y - n/2;
        let mask = (1 << n) - 1;
        
        //dev_log!("checking conflict for move:{:?}, piece: {:?}", m, p);
        for y in 0..n {
            // The bits representing a single row of the piece map
            let shift: u8 = (n * (n - 1 - y)) as u8;
            let bitseg: u16 = reverse_bin( (( map & (mask << shift) ) >> shift) as u16 , n as u8 );
            //dev_log!("c_x: {c_x}, map: {:#011b}, bitseg: {:#07b}", PIECE_MAP[*p as usize][m.r as usize], bitseg);

            // If empty row on piece map
            if bitseg == 0 {
                continue;
            }
            // If out of board on upper edge
            if  c_y + y < 0 {
                //continue;
                return true;
            }
            // If out of board on bottom edge
            if c_y + y >= 20 {
                return true
            }
            // If out of board on left edge
            if c_x < 0 && bitseg & ((1 << (-c_x)) - 1) > 0  {
                return true
            }
            // Shift according to c_x
            let bitseg = if c_x > 0 { bitseg << c_x } else { bitseg >> -c_x };

            // If out of board on right edge
            if  bitseg > (1 << 10) -1 {
                return true
            }

            if self.m[(c_y + y) as usize] & bitseg > 0 {
                return true
            }
        };
        false
    }   
    /*
        Pastes a given piece onto a clone of self according to given Move, returning said clone.
     */
    pub fn apply_move (self: &Self, m: &Move, piece: &Piece, hold: &Piece) -> Field {
        let mut field = self.clone();
        let p: &Piece = if m.hold {hold} else {piece};
        let map: &u32 = &PIECE_MAP[*p as usize][m.r as usize];
        let n: i8 = if *p == Piece::I {5} else {3};
        let c_x: i8 = m.x - n/2;
        let c_y: i8 = m.y - n/2;
        let mask = (1 << n) - 1;
        
        //dev_log!("applying move:{:?}, piece:{:?}", m, p);
        for y in 0..n {
            // The bits representing a single row of the piece map
            let shift: u8 = (n * (n - 1 - y)) as u8;
            let bitseg: u16 = reverse_bin( (( map & (mask << shift) ) >> shift) as u16 , n as u8 );
            //dev_log!("c_x: {c_x}, map: {:09b}, bitseg: {:05b}", PIECE_MAP[*p as usize][m.r as usize], bitseg);

            // If empty row on piece map
            if bitseg == 0 {
                continue;
            }
            // If out of board on upper edge
            if  c_y + y < 0 {
                panic!("@ Field.apply_move: out of board on upper edge");
            }
            // If out of board on bottom edge
            if c_y + y >= 20 {
                panic!("@ Field.apply_move: out of board on bottom edge");
            }
            // If out of board on left edge
            if c_x < 0 && bitseg & ((1 << (-c_x)) - 1) > 0  {
                panic!("@ Field.apply_move: out of board on left edge");
            }
            // Shift according to c_x
            let bitseg = if c_x > 0 { bitseg << c_x } else { bitseg >> -c_x };
            //dev_log!("c_x: {}, final bitseg: {:05b}", c_x, bitseg);
            // If out of board on right edge
            if bitseg > (1 << 10)-1 {
                panic!("@ Field.apply_move: out of board on right edge");
            }
            field.m[(c_y + y) as usize] |= bitseg;
        };
        //dev_log!("{}", field);
        field
    }
    /*
        Sets a Prop object by processing a pasted field. This necesitates some info from Move object
     */
    pub fn set_props (self: &mut Self, mov: &Move, props: &mut Props) {
        // Clear rows
        let mut clears: usize = 0;
        for y in (0..20).rev() {
            if clears > 0 {
                self.m[y+clears] = self.m[y];
            }
            if self.m[y] == (1 << 10) - 1 {
                clears += 1;
            }
            if clears > 0 {
                self.m[y] = 0;
            }
        }
        // Calc attacks 
        let atk: u8 = match clears {
            1 => if mov.tspin {2} else {0},
            2 => if mov.tspin {4} else {1},
            3 => if mov.tspin {6} else {2},
            4 => 4,
            _ => 0,
        };

        // Setting attacks & ds (clears)
        props.atk = atk;
        props.ds = clears as u8;

        // If perfect clear
        if clears > 0 && self.m.iter().sum::<u16>() == 0 {
            props.atk += 10;
        }

        // Combo
        props.combo = if clears > 0 {props.combo + 1}  else {0};
        
        // b2b
        props.b2b = if (mov.tspin && clears > 0) || clears == 4 {props.b2b + 1} else {0};
    }
}

pub fn reverse_bin (mut x: u16, n: u8) -> u16 {
    let mut r: u16 = 0;
    for _ in 0..n {
        r <<= 1;
        r ^= x & 1;
        x >>= 1;
    }
    r
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

#[cfg(test)]
mod test {
    use super::*;
    use crate::Key;
    
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
    fn field_apply_move_test () {
        let mut field: Field = Field::new();
        field.m = [
            0b0_0_0_0_0_0_0_0_0_0,
            0b0_0_0_0_0_0_0_0_0_0,
            0b0_0_0_0_0_0_0_0_0_0,
            0b0_0_0_0_0_0_0_0_0_0,
            0b0_0_0_0_0_0_0_0_0_0,
            0b0_0_0_0_0_0_0_0_0_0,
            0b0_0_0_0_0_0_0_0_0_0,
            0b0_0_0_0_0_0_0_0_0_0,
            0b0_0_0_0_0_0_0_0_0_0,
            0b0_0_0_0_0_0_0_0_0_0,
            0b0_0_0_0_0_0_0_0_0_0,
            0b0_0_0_0_0_0_0_0_0_0,
            0b0_0_0_0_0_0_0_0_0_0,
            0b0_0_0_0_0_0_0_0_0_0,
            0b0_0_0_0_0_0_0_0_0_0,
            0b0_0_0_0_0_0_0_0_0_0,
            0b0_0_0_0_0_0_0_0_0_0,
            0b0_0_0_0_0_0_0_0_0_0,
            0b0_0_0_0_0_0_0_0_0_1,
            0b0_0_0_0_0_0_0_1_1_1,
        ];
        let mut mov: Move = Move::new();
        mov.x = 9;
        mov.y = 17;
        mov.r = 1;
        mov.hold = true;

        let p: Piece = Piece::Z;
        let h: Piece = Piece::I;

        //mov.apply_key(&Key::Hold, &field, &p, &h);
        //mov.apply_key(&Key::Cw, &field, &p, &h);
        //mov.apply_key(&Key::Left, &field, &p, &h);
        //mov.apply_key(&Key::Left, &field, &p, &h);
        //mov.apply_key(&Key::Left, &field, &p, &h);
        //mov.apply_key(&Key::Left, &field, &p, &h);
        //mov.apply_key(&Key::HardDrop, &field, &p, &h);

        field = field.apply_move(&mov, &p, &h);

        /*
        assert_eq!(field.m[17], 0b00000_00000);
        assert_eq!(field.m[18], 0b00001_10000);
        assert_eq!(field.m[19], 0b00001_10000);
         */
    }

    #[test] 
    fn field_set_props_test () {
        let mut m = Move::new();
        let mut props = Props::new();
        let mut field = Field::new();
        
        field.m = [
            0b0_0_0_0_0_0_0_0_0_0,
            0b0_0_0_0_0_0_0_0_0_0,
            0b0_0_0_0_0_0_0_0_0_0,
            0b0_0_0_0_0_0_0_0_0_0,
            0b0_0_0_0_0_0_0_0_0_0,
            0b0_0_0_0_0_0_0_0_0_0,
            0b0_0_0_0_0_0_0_0_0_0,
            0b0_0_0_0_0_0_0_0_0_0,
            0b0_0_0_0_0_0_0_0_0_0,
            0b0_0_0_0_0_0_0_0_0_0,
            0b0_0_0_0_0_0_0_0_0_0,
            0b0_0_0_0_0_0_0_0_0_0,
            0b0_0_0_0_0_0_0_0_0_0,
            0b0_0_0_0_0_0_0_0_0_0,
            0b0_0_0_0_0_0_0_0_0_0,
            0b0_0_0_0_0_0_0_0_0_0,
            0b0_0_0_0_0_0_0_0_0_0,
            0b0_0_0_0_0_0_0_0_0_0,
            0b1_1_1_1_0_0_1_1_1_1,
            0b1_1_1_1_0_0_1_1_1_1,
        ];
         
        m.apply_key(&Key::HardDrop, &field, &Piece::O, &Piece::O);

        field = field.apply_move(&m, &Piece::O, &Piece::O);
        field.set_props(&m, &mut props);

        assert_eq!(props.sum_ds, 2);
        assert_eq!(field.m[18], 0);
        assert_eq!(field.m[19], 0);
    }
}