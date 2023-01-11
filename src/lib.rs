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
pub use evaluator::{evaluate, EvaluatorMode};


pub mod mac {
    macro_rules! dev_log {
        ($s:literal) => {
            if std::env::var("LOG").is_ok() {
                print!($s);
            }
        };
        (ln, $s:literal) => {
            if std::env::var("LOG").is_ok() {
                println!($s);
            }
        };
        ($s:literal, $($a: expr),* ) => {
            if std::env::var("LOG").is_ok() {
                print!(
                    $s,
                    $($a,)*
                );
            }
        };
        (ln, $s:literal, $($a: expr),* ) => {
            if std::env::var("LOG").is_ok() {
                println!(
                    $s,
                    $($a,)*
                );
            }
        };
    }
    pub(crate) use dev_log;
}

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

#[derive(Copy, Clone, Debug)]
pub struct Props {
    pub sum_atk: u8,
    pub sum_ds: u8,
    pub sum_no_atk: u8, // sum of atk-less ds.
    pub atk: u8,
    pub ds: u8,
    pub b2b: u8,
    pub combo: u8,
    pub clears: u32,
}
impl Props {
    pub fn new () -> Self {
        Self {
            b2b: 0,
            combo: 0,
            sum_atk: 0,
            sum_ds: 0,
            sum_no_atk: 0,
            atk: 0,
            ds: 0,
            clears: 0,
        }
    }
}