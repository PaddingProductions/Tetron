use super::{Piece, Move, Props};

use std::fmt;

pub static mut COUNTER: u128 = 0;
/// Effective allias for `[u16; 20]`, representing the game board.
/// 
/// Minial memory footprint.
/// Implements getting, setting, and helper functions.
#[derive(PartialEq, Eq, Hash, Clone)]
pub struct Field {
    pub m: [u16; 20],
}
pub type ConflictCache = [[u32; 20]; 4];
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

// (c_x, c_y) denotes the corner of the map. This is what is typically used in computation
// (x, y) denotes the center of the map. This is what is stored in Moves
impl Field {
    pub fn new () -> Self {
        Self {
            m: [0; 20],
        }
    }

    pub fn check_conflict(&self, cache: &mut ConflictCache, m: &Move, p: &Piece) -> bool {
        let _bencher: Option<crate::Bencher> = if cfg!(feature = "bench") {
            unsafe {
                Some( crate::Bencher::new( &mut crate::BENCH_DATA.conflict ) )
            }
        } else {None};

       if m.y < 0 || m.y >= 20 || m.x < 0 || m.x >= 10 {
            return true;
        }
        if cache[m.r as usize][m.y as usize] & 1 << (10 + m.x as usize) == 0 {
            cache[m.r as usize][m.y as usize] |= 1 << (10 + m.x as usize);
            if self.compute_conflict(m, p) {
                cache[m.r as usize][m.y as usize] |= 1 << (m.x as usize);
            }
        } 
        cache[m.r as usize][m.y as usize] & 1 << (m.x as usize) > 0
    }

    fn compute_conflict (&self, m: &Move, p: &Piece) -> bool {
        let map: &[u16; 5] = &PIECE_MAP[*p as usize][m.r as usize];
        let n: i8 = if *p == Piece::I {5} else {3};
        let c_x: i8 = m.x - n/2;
        let c_y: i8 = m.y - n/2;
        
        //dev_log!("checking conflict for move:{:?}, piece: {:?}", m, p);
        for y in 0..n {
            // The bits representing a single row of the piece map
            let bitseg: u16 = map[y as usize].reverse_bits() >> (16 - n);
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

    /// Pastes a given piece onto a clone of self according to given move, returning said clone.
    pub fn apply_move (self: &Self, m: &Move, piece: &Piece, hold: &Piece) -> Result<Field, ()> {
        let mut field = self.clone();
        let p: &Piece = if m.hold {hold} else {piece};
        let map: &[u16; 5] = &PIECE_MAP[*p as usize][m.r as usize];
        let n: i8 = if *p == Piece::I {5} else {3};
        let c_x: i8 = m.x - n/2;
        let c_y: i8 = m.y - n/2;
        
        //dev_log!("applying move:{:?}, piece:{:?}", m, p);
        for y in 0..n {
            // The bits representing a single row of the piece map
            let bitseg: u16 = map[y as usize].reverse_bits() >> (16 - n);
            //dev_log!("c_x: {c_x}, map: {:09b}, bitseg: {:05b}", PIECE_MAP[*p as usize][m.r as usize], bitseg);

            // If empty row on piece map
            if bitseg == 0 {
                continue;
            }
            // If out of board on upper edge
            if  c_y + y < 0 {
                //return Err(());
                panic!("@ Field.apply_move: out of board on upper edge");
            }
            // If out of board on bottom edge
            if c_y + y >= 20 {
                //return Err(());
                panic!("@ Field.apply_move: out of board on bottom edge");
            }
            // If out of board on left edge
            if c_x < 0 && bitseg & ((1 << (-c_x)) - 1) > 0  {
                //return Err(());
                panic!("@ Field.apply_move: out of board on left edge");
            }
            // Shift according to c_x
            let bitseg = if c_x > 0 { bitseg << c_x } else { bitseg >> -c_x };
            //dev_log!("c_x: {}, final bitseg: {:05b}", c_x, bitseg);
            // If out of board on right edge
            if bitseg > (1 << 10)-1 {
                //return Err(());
                panic!("@ Field.apply_move: out of board on right edge");
            }
            field.m[(c_y + y) as usize] |= bitseg;
        };
        //dev_log!("{}", field);
        Ok(field)
    }

    /// Processes self after a move is pasted. Writes attributes into `Prop` object.
    ///
    /// Clears lines. 
    /// Calculates attacks.
    /// This necesitates some info from `Move` object, thus the parameter.
    pub fn set_props (self: &mut Self, mov: &Move, props: &mut Props) {
        // Clear rows
        let mut clears: usize = 0;
        for y in (0..20).rev() {
            if clears > 0 {
                self.m[y+clears] = self.m[y];
            }
            if self.m[y] == (1 << 10) - 1 {
                props.clears += 1 << y;
                clears += 1;
            }
            if clears > 0 {
                self.m[y] = 0;
            }
        }
        // Calc attacks 
        let atk: u8 = if clears < 4 && !mov.tspin {
            match clears {
                0 => 0,
                1 => [0, 0, 1, 1, 1, 1, 2, 2, 2, 2][props.combo as usize],
                2 => [1, 1, 1, 1, 2, 2, 2, 2, 3, 3][props.combo as usize],
                3 => [2, 2, 3, 3, 4, 4, 5, 5, 6, 6][props.combo as usize],
                _ => 0
            }
        } else if clears > 0 {
            let t = if mov.tspin {clears} else {0};
            B2B_TABLE[props.b2b as usize][t][props.combo as usize] as u8
        } else {0};

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

/// Reverses the parameter's binary representation, given the width.
pub fn reverse_bin (mut x: u16, n: u8) -> u16 {
    let mut r: u16 = 0;
    for _ in 0..n {
        r <<= 1;
        r ^= x & 1;
        x >>= 1;
    }
    r
}

/// Attack table, ripped from Tetr.io
// <TODO>: T-spin Minis
pub const B2B_TABLE: [[[u32; 10]; 4]; 4] = [
    [
        [4, 5, 6, 7, 8, 9, 10, 11, 12, 13],
        [2, 2, 3, 3, 4, 4, 5, 5, 6, 6],
        [4, 5, 6, 7, 8, 9, 10, 11, 12, 13],
        [6, 7, 9, 10, 12, 13, 15, 16, 18, 19],
    ],
    [
        [5, 6, 7, 8, 10, 11, 12, 13, 15, 16],
        [3, 3, 4, 5, 6, 6, 7, 8, 9, 9],
        [5, 6, 7, 8, 10, 11, 12, 13, 15, 16],
        [7, 8, 10, 12, 14, 15, 17, 19, 21, 22],
    ],
    [
        [6, 7, 9, 10, 12, 13, 15, 16, 18, 19],
        [4, 5, 6, 7, 8, 9, 10, 11, 12, 13],
        [6, 7, 9, 10, 12, 13, 15, 16, 18, 19],
        [8, 10, 12, 14, 16, 18, 20, 22, 24, 25],
    ],
    [
        [7, 8, 10, 12, 14, 15, 17, 19, 21, 22],
        [5, 6, 7, 8, 10, 11, 12, 13, 15, 16],
        [7, 8, 10, 12, 14, 15, 17, 19, 21, 22],
        [9, 11, 13, 15, 18, 20, 22, 24, 27, 29],
    ],
];

/// Binary representation of piece shapes.
///
/// Visually inversed, due to bit order.
pub const PIECE_MAP: [[[u16; 5]; 4]; 7] = [
    [ // J
        [0b100, 0b111, 0b000, 0, 0],
        [0b011, 0b010, 0b010, 0, 0],
        [0b000, 0b111, 0b001, 0, 0],
        [0b010, 0b010, 0b110, 0, 0]
    ],
    [ // L
        [0b001, 0b111, 0b000, 0, 0],
        [0b010, 0b010, 0b011, 0, 0],
        [0b000, 0b111, 0b100, 0, 0],
        [0b110, 0b010, 0b010, 0, 0]
    ], 
    [ // S
        [0b011, 0b110, 0b000, 0, 0],
        [0b010, 0b011, 0b001, 0, 0],
        [0b000, 0b011, 0b110, 0, 0],
        [0b100, 0b110, 0b010, 0, 0]
    ], 
    [ // Z
        [0b110, 0b011, 0b000, 0, 0],
        [0b001, 0b011, 0b010, 0, 0],
        [0b000, 0b110, 0b011, 0, 0],
        [0b010, 0b110, 0b100, 0, 0]
    ], 
    [ // T
        [0b010, 0b111, 0b000, 0, 0],
        [0b010, 0b011, 0b010, 0, 0],
        [0b000, 0b111, 0b010, 0, 0],
        [0b010, 0b110, 0b010, 0, 0]
    ], 
    [ // I
        [0b00000, 0b00000, 0b01111, 0b00000, 0b00000],
        [0b00000, 0b00100, 0b00100, 0b00100, 0b00100],
        [0b00000, 0b00000, 0b00000, 0b01111, 0b00000],
        [0b00100, 0b00100, 0b00100, 0b00100, 0b00000]
    ],
    [ // O
        [0b011, 0b011, 0b000, 0, 0],
        [0b000, 0b011, 0b011, 0, 0],
        [0b000, 0b110, 0b110, 0, 0],
        [0b110, 0b110, 0b000, 0, 0]
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
        let mut cache: ConflictCache = [[0; 20]; 4]; 

        mov.y = 19;
        mov.x = 0;
        assert_eq!(field.check_conflict(&mut cache, &mov, &p), true);
    }

    #[test]
    fn field_conflict_map_test () {
         
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
            0b1_1_1_1_1_0_0_0_0_0,
            0b1_1_1_1_1_1_1_0_0_0,
        ];
        println!("Field:\n{}", field);

        let mut cache: ConflictCache = [[0; 20]; 4]; 

        for r in 0..4 {
            println!("orientation: {r}\n");
            for y in 0..20 {
                for x in 0..10 {
                    let m = Move {
                        x,
                        y,
                        r, 
                        s: -1,
                        list: 0, 
                        hold: false,
                        tspin: false,
                        lock: false
                    };
                    if field.m[y as usize] & 1 << x > 0 {
                        print!("# ");
                    } else {
                        print!("{} ", if field.check_conflict(&mut cache, &m, &Piece::T) { 'x' } else { '.' });
                    }
                }
                println!();
            }
        }
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
            0b1_1_1_1_1_0_0_0_0_0,
            0b1_1_1_1_1_1_1_0_0_0,
        ];


        let mut mov: Move = Move::new();
        let p: Piece = Piece::L;
        let h: Piece = Piece::L;
        let mut cache: (ConflictCache, ConflictCache) = ([[0; 20]; 4], [[0; 20]; 4]);

        mov.apply_key(&Key::Cw, &mut cache, &field, &p, &h);
        mov.apply_key(&Key::Left, &mut cache, &field, &p, &h);
        mov.apply_key(&Key::Left, &mut cache, &field, &p, &h);
        mov.apply_key(&Key::Left, &mut cache, &field, &p, &h);
        mov.apply_key(&Key::Left, &mut cache, &field, &p, &h);
        //mov.apply_key(&Key::SoftDrop, conflict_cache, &field, &p, &h);
        //mov.apply_key(&Key::Ccw, conflict_cache, &field, &p, &h);
        mov.apply_key(&Key::HardDrop, &mut cache, &field, &p, &h);

        println!("{:?}", mov);
        field = field.apply_move(&mov, &p, &h).unwrap();
        println!("{}", field);
        
        //assert_eq!(field.m[17], 0b00000_00000);
        //assert_eq!(field.m[18], 0b00001_10000);
        //assert_eq!(field.m[19], 0b00001_10000);
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
            0b1_1_1_1_1_0_0_0_0_0,
            0b1_1_1_1_1_1_1_0_0_0,
        ];
        let cache: ConflictCache = [[0; 20]; 4]; 

        m.apply_key(&Key::HardDrop, &mut (cache, cache), &field, &Piece::L, &Piece::L);

        field = field.apply_move(&m, &Piece::O, &Piece::O).unwrap();
        println!("{}", field);
        field.set_props(&m, &mut props);
        println!("{}", field);

        assert_eq!(props.ds, 2);
        assert_eq!(field.m[18], 0);
        assert_eq!(field.m[19], 0);
    }
}
