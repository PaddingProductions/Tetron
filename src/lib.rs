//#![warn(missing_docs)]

//! _Tetron_ - A Tetris Bot in Rust for the multiplayer ruleset
//!
//! Uses trivial DFS for exploration and BFS on keystrokes for placements.
//! Written in Rust for memory safety and fast allocation speeds.
//!
//! Example:
//! ```
//! let state: tetron::State = tetron::State::new();
//! // .. set state
//! let out: Option<(tetron::State, tetron::Move, f32)> = tetron::solve(&state, &tetron::config::Config::new(2, tetron::EvaluatorMode::Norm));
//! ```

pub mod solve;
pub mod gen_moves;
pub mod field;
pub mod state;
pub mod mov;
pub mod evaluator;
pub mod mac;
pub mod config;

pub use field::Field;
pub use state::State;
pub use mov::Move;
pub use gen_moves::gen_moves;
pub use solve::solve;
pub use evaluator::{evaluate, EvaluatorMode};

// WASM CONSOLE
// use wasm_bindgen::prelude::*;
// use web_sys::console;

/*
#[macro_export]
macro_rules! console_log {
    ($($arg: expr), *) => {
        console::log_1(
            &JsValue::from_str(
                &format!(
                    $( $arg, )*
                )
            )
        );
    }
}
*/

/// Enumeration representing Tetris Piece types.
#[derive(PartialEq, Eq, Hash, Copy, Clone, Debug)]
#[repr(u8)]
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

/// Enumeration representing possible keystrokes.
#[repr(u8)]
#[derive(PartialEq, Clone, Debug)]
pub enum Key {
    Left,
    Right,
    DASLeft,
    DASRight,
    Cw,
    Ccw,
    _180,
    HardDrop,
    SoftDrop,
    Hold,
}

/// Minimalist structure containing properties of a state.
/// 
/// Concentrates all attributes of a given state into one object.
///
/// Non-intuitive attributes:
/// `sum_no_atk`: Downstack lines without an attack.
#[derive(Copy, Clone, Debug, Hash)]
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

pub struct Bencher <'a> {
    start: std::time::Instant,
    data: &'a mut(u128, u128)
}

impl<'a> Bencher<'a> {
    fn new (data: &'a mut(u128, u128)) -> Self {
        Self { 
            start: std::time::Instant::now(),
            data: data
        }
    }
}
impl<'a> Drop for Bencher<'a> {
    fn drop (&mut self) {
        let dt = self.start.elapsed().as_nanos();
        self.data.0 = (self.data.0 * self.data.1 + dt) / (self.data.1+1);
        self.data.1 += 1;
    }
}

struct BenchData {
    solves: u128,
    evaluator: (u128, u128),
    gen_moves: (u128, u128),
    apply_key: (u128, u128),
    conflict: (u128, u128),
    solve_d0: (u128, u128),
}
static mut BENCH_DATA: BenchData = BenchData {
    solves: 0,
    evaluator: (0, 0),
    gen_moves: (0, 0),
    apply_key: (0, 0),
    conflict: (0, 0),
    solve_d0: (0, 0),
};
/// Resets globally stored bench data. Developer Tool.
///
/// Unsafe function reseting data for development bench tests.
/// Should only be used in development.
pub fn bench_reset () {
    unsafe {
        BENCH_DATA = BenchData {
            solves: 0,
            evaluator: (0, 0),
            gen_moves: (0, 0),
            apply_key: (0, 0),
            conflict: (0, 0),
            solve_d0: (0, 0),
        };
    }
}
/// Increments solve counter. Developer Tool.
///
/// Unsafe function incrementing a vital counter in analyzing bench data. 
/// Should only be used in development.
pub fn bench_increment_solve () {
    unsafe {
        BENCH_DATA.solves += 1;
    }
}

/// Prints bench data. Developer Tool.
///
/// Unsafe as it interacts with globally exposed bench data.
/// Should only be used in development.
pub fn print_bench_result () { unsafe {

    let d = &BENCH_DATA;
    println!("=== Bench Result ==="); 
    {let t = d.evaluator; println!("evaluator: avg dt: {}, cnt: {}, total dt: {:.3}", t.0, t.1, (t.0*t.1/d.solves) as f64/1000.0);}
    {let t = d.gen_moves; println!("gen_moves: avg dt: {}, cnt: {}, total dt: {:.3}", t.0, t.1, (t.0*t.1/d.solves) as f64/1000.0);}
    {let t = d.apply_key; println!("apply_key: avg dt: {}, cnt: {}, total dt: {:.3}", t.0, t.1, (t.0*t.1/d.solves) as f64/1000.0);}
    {let t = d.conflict; println!("conflict: avg dt: {}, cnt: {}, total dt: {:.3}", t.0, t.1, (t.0*t.1/d.solves) as f64/1000.0);}
    {let t = d.solve_d0; println!("solve_d0: avg dt: {}, cnt: {}, total dt: {:.3}", t.0, t.1, (t.0*t.1/d.solves) as f64/1000.0);}
}}
