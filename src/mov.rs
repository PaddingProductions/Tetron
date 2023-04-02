use std::time::Instant;

use crate::BENCH_DATA;
use super::{Key, Piece, Field};

/// Minimalist structure containing properties of a piece placement.
///
/// Optimized memory usage to minimize memory allocation penalty.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Move {
    pub x: i8,
    pub y: i8,
    pub r: u8,
    pub s: i8, // -1 for none, -2 if just softdropped, should copy on next spin.
    pub tspin: bool,
    pub hold: bool,
    pub lock: bool
}

impl Move {
    /// Spanws new `Move` instance.
    pub fn new () -> Self {
        Self {
            x: 4,
            y: 1,
            r: 0,
            s: -1, 
            tspin: false,
            hold: false,
            lock: false,
        }
    }
    ///  Function managing spins & kicks.
    ///
    ///  Behavior in accordance with the SRS kicktable.
    ///  Returns boolean representing if the spin succeeded.
    fn apply_spin (self: &mut Self, field: &Field, p: &Piece, d: &i8) -> bool {
        let r = self.r as usize;
        let nr = (self.r as i8 + d).rem_euclid(4) as usize;
        let table_ref: &[[(i8, i8); 5]; 4] = match *p {
            Piece::I => &KICK_TABLE_I,
            _ => &KICK_TABLE
        };

        let kicks: Vec<(i8, i8)> = (0..5).map(
            |i| (
                table_ref[r][i].0 - table_ref[nr][i].0,
                table_ref[r][i].1 - table_ref[nr][i].1
            )).collect();
        
        self.r = nr as u8;
        for i in 0..5 {
            self.x += kicks[i].0;
            self.y -= kicks[i].1;
            if !field.check_conflict(&*self, p) {
                // Check t-spin
                if *p == Piece::T {
                    // Three-corner rule
                    // TODO: differentiate mini-tspins and tspins :p
                    let cnt: u8 = 
                        [(self.x-1, self.y-1), (self.x-1, self.y+1), (self.x+1, self.y-1), (self.x+1 , self.y+1)]
                        .map(|(x, y)| 
                            if x < 0 || y < 0 || x >= 10 || y >= 20 || field.m[y as usize] & (1 << x) > 0 {1 as u8} else {0 as u8}
                        ).iter().sum::<u8>();
                    self.tspin = cnt >= 3;
                }
                return true
            }
            self.x -= kicks[i].0;
            self.y += kicks[i].1;
        }
        self.r = r as u8;
        false
    }
    
    // Applies keystroke to self, altering attributes.
    //
    // Returns whether the key altered the attributes.
    pub fn apply_key(self: &mut Self, key: &Key, field: &Field, piece: &Piece, hold: &Piece) -> bool {
        if cfg!(feature = "bench") {
            let start = Instant::now();
            defer!(unsafe {
                BENCH_DATA.apply_key.1 += 1;
                let dt = start.elapsed().as_micros();
                BENCH_DATA.apply_key.0 = if BENCH_DATA.apply_key.0 == 0 {dt} else {(BENCH_DATA.apply_key.0 + dt) / 2};
            });
        }
        
        let p: &Piece = if self.hold {hold} else {piece};

        match key {
            Key::Left | Key::Right => {
                let d: i8 = if *key == Key::Left {-1} else {1};
                self.x += d;

                if field.check_conflict(&*self, p) {
                    self.x -= d;
                    return false;
                }
            }, 
            Key::Cw | Key::Ccw | Key::_180 => {
                if self.s == -2 {
                    self.s = self.r as i8;
                }
                    
                let d: i8 = if *key == Key::Cw {1} else if *key == Key::Ccw {-1} else {2};
                
                if !self.apply_spin(field, p, &d) {
                    return false;
                }
            }, 
            Key::DASLeft => {
                while self.apply_key(&Key::Left, field, piece, hold) {}
            }, 
            Key::DASRight => {
                while self.apply_key(&Key::Right, field, piece, hold) {}
            }, 
            Key::SoftDrop => {
                while !field.check_conflict(&*self, p) {
                    self.y += 1;
                }
                self.y -= 1;
                self.s = -2; // Read comment on declaration. Spin tracking.
            },
            Key::HardDrop => {
                while !field.check_conflict(&*self, p) {
                    self.y += 1;
                }
                self.y -= 1;
                self.lock = true;
            }
            Key::Hold => {
                if self.hold {
                    return false;
                }
                self.hold = true
            }
        }; 
        true
    }
}

const KICK_TABLE: [[(i8, i8); 5]; 4] = [
    [( 0, 0), ( 0, 0), ( 0, 0), ( 0, 0), ( 0, 0)],
    [( 0, 0), ( 1, 0), ( 1,-1), ( 0, 2), ( 1, 2)],
    [( 0, 0), ( 0, 0), ( 0, 0), ( 0, 0), ( 0, 0)],
    [( 0, 0), (-1, 0), (-1,-1), ( 0, 2), (-1, 2)],
];

const KICK_TABLE_I: [[(i8, i8); 5]; 4] = [
    [( 0, 0), (-1, 0), ( 2, 0), (-1, 0), ( 2, 0)],
    [(-1, 0), ( 0, 0), ( 0, 0), ( 0, 1), ( 0,-2)],
    [(-1, 1), ( 1, 1), (-2, 1), ( 1, 0), (-2, 0)],
    [( 0, 1), ( 0, 1), ( 0, 1), ( 0,-1), ( 0, 2)],
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test] 
    fn move_apply_key_test () {
        let field: Field = Field::new();
        let mut mov: Move = Move::new();
        let p: Piece = Piece::I;
        let h: Piece = Piece::J;

        //mov.apply_key(&Key::Cw, &field, &p, &h);
        mov.apply_key(&Key::Right, &field, &p, &h);
        mov.apply_key(&Key::Right, &field, &p, &h);
        mov.apply_key(&Key::Right, &field, &p, &h);
        mov.apply_key(&Key::Right, &field, &p, &h);
        //mov.apply_key(&Key::Left, &field, &p, &h);

        mov.apply_key(&Key::HardDrop, &field, &p, &h);

        println!("move: {:?}", mov);
        //assert_eq!(mov.x, 1);
        //assert_eq!(mov.y, 18);
        //assert_eq!(mov.r, 1);
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
}
