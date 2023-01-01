use super::*;

#[derive(Clone, Debug)]
pub struct Move {
    pub x: i8,
    pub y: i8,
    pub r: u8,
    pub s: u8,
    pub kick: bool,
    pub hold: bool
}

impl Move {
    pub fn new () -> Self {
        Self {
            x: 4,
            y: 1,
            r: 0,
            s: 0,
            kick: false,
            hold: false,
        }
    }
    /*
        Handles the modular arithmetic involved with spining.
     */
    fn set_spin (self: &mut Self, d: &i8) {
        self.r = (self.r as i8 + d).rem_euclid(4) as u8;
    }
    /* 
        Handles kicks
        Returns if spin passed
     */
    fn apply_spin (self: &mut Self, field: &Field, p: &Piece, d: &i8) -> bool {
        self.set_spin(d);
        if field.check_conflict(&*self, p) {
            self.set_spin(&-d);
            return false
        }
        true
    }
    /*
        Changes attributes in self based on given Key
        Returns whether the key affects the attributes 
     */
    pub fn apply_key(self: &mut Self, key: &Key, field: &Field, piece: &Piece, hold: &Piece) -> bool {
        
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
                let d: i8 = if *key == Key::Cw {1} else if *key == Key::Ccw {-1} else {2};
                
                if !self.apply_spin(field, p, &d) {
                    return false;
                }
            }, 
            Key::HardDrop => {
                while !field.check_conflict(&*self, p) {
                    self.y += 1;
                }
                self.y -= 1;
            }
            Key::Hold => {
                if self.hold {
                    return false;
                }
                self.hold = true
            }
            _ => return false,
        }; 
        true
    }
}