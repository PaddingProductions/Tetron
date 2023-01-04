pub mod solve;
pub mod gen_moves;
pub mod field;
pub mod state;
pub mod mov;
pub mod evaluator;

pub use field::Field;
pub use state::State;
pub use mov::Move;
pub use gen_moves::gen_moves;
pub use solve::solve;
pub use evaluator::evaluate;

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

#[derive(Copy, Clone, PartialEq, Debug)]
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

}
