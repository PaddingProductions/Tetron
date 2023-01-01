use super::*;

fn reverse_bin (mut x: u16, n: u8) -> u16 {
    let mut r: u16 = 0;
    for _ in 0..n {
        r <<= 1;
        r ^= x & 1;
        x >>= 1;
    }
    r
}

#[derive(PartialEq, Eq, Hash, Copy, Clone)]
pub struct Field {
    pub m: [u16; 20],
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

        //println!("move:{:?}", m);
        for y in 0..n {
            // The bits representing a single row of the piece map
            let shift: u8 = (n * (2 - y)) as u8;
            let bitseg: u16 = reverse_bin( (( map & (0b111 << shift) ) >> shift) as u16 , n as u8 );
            //println!("c_x: {c_x}, map: {:#011b}, bitseg: {:#07b}", PIECE_MAP[*p as usize][m.r as usize], bitseg);

            // If empty row on piece map
            if bitseg == 0 {
                continue;
            }
            // If out of board on upper edge
            if  c_y + y < 0 {
                return true;
            }
            // If out of board on bottom edge
            if c_y + y >= 20 {
                return true
            }
            // If out of board on left edge
            if c_x < 0 && bitseg & (1 << (-c_x) - 1) > 0  {
                return true
            }
            // Shift according to c_x
            let bitseg = if c_x > 0 { bitseg << c_x } else { bitseg >> -c_x };

            // If out of board on right edge
            if  bitseg > (1 << 10) {
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

        //println!("move:{:?}", m);
        //println!("c_y:{c_y}, p:{}", *p as usize);
        for y in 0..n {
            // The bits representing a single row of the piece map
            let shift: u8 = (n * (2 - y)) as u8;
            let bitseg: u16 = reverse_bin( (( map & (0b111 << shift) ) >> shift) as u16 , n as u8);
            //println!("c_x: {c_x}, map: {:#011b}, bitseg: {:#07b}", PIECE_MAP[*p as usize][m.r as usize], bitseg);

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
            if c_x < 0 && bitseg & (1 << (-c_x) - 1) > 0  {
                panic!("@ Field.apply_move: out of board on left edge");
            }
            // Shift according to c_x
            let bitseg = if c_x > 0 { bitseg << c_x } else { bitseg >> -c_x };
            
            // If out of board on right edge
            if bitseg > (1 << 10) {
                panic!("@ Field.apply_move: out of board on right edge");
            }
            field.m[(c_y + y) as usize] |= bitseg;
        };
        field
    }
    /*
        Sets a Prop object by processing a pasted field. This necesitates some info from Move object
     */
    pub fn set_props (self: &mut Self, mov: &Move, props: &mut Props) {

    }
}
