pub mod solve;
pub mod gen_moves;
pub mod field;
pub mod state;
pub mod mov;
pub mod evaluator;
pub mod mac;

pub use field::Field;
pub use state::State;
pub use mov::Move;
pub use gen_moves::gen_moves;
pub use solve::solve;
pub use evaluator::{evaluate, EvaluatorMode};

#[macro_use(defer)]
extern crate scopeguard;

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

#[derive(PartialEq, Clone)]
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

struct BenchData {
    solves: u128,
    evaluator: (u128, u128),
    gen_moves: (u128, u128),
    apply_key: (u128, u128),
    solve_d0: (u128, u128),
}
static mut BENCH_DATA: BenchData = BenchData {
    solves: 0,
    evaluator: (0, 0),
    gen_moves: (0, 0),
    apply_key: (0, 0),
    solve_d0: (0, 0),
};
pub fn bench_reset () {
    unsafe {
        BENCH_DATA = BenchData {
            solves: 0,
            evaluator: (0, 0),
            gen_moves: (0, 0),
            apply_key: (0, 0),
            solve_d0: (0, 0),
        };
    }
}
pub fn bench_increment_solve () {
    unsafe {
        BENCH_DATA.solves += 1;
    }
}
pub fn print_bench_result () { unsafe {

    let d = &BENCH_DATA;
    println!("=== Bench Result ==="); 
    {let t = d.evaluator; println!("evaluator: avg dt: {}, cnt: {}, total dt: {:.3}", t.0, t.1, (t.0*t.1/d.solves) as f64/1000.0);}
    {let t = d.gen_moves; println!("gen_moves: avg dt: {}, cnt: {}, total dt: {:.3}", t.0, t.1, (t.0*t.1/d.solves) as f64/1000.0);}
    {let t = d.apply_key; println!("apply_key: avg dt: {}, cnt: {}, total dt: {:.3}", t.0, t.1, (t.0*t.1/d.solves) as f64/1000.0);}
    {let t = d.solve_d0; println!("solve_d0: avg dt: {}, cnt: {}, total dt: {:.3}", t.0, t.1, (t.0*t.1/d.solves) as f64/1000.0);}
}}
