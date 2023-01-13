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

struct BenchData {
    evaluator: (u128, u128),
    check_conflict: (u128, u128),
    apply_move: (u128, u128),
    gen_moves: (u128, u128),
    apply_key: (u128, u128),
    solve_d0: (u128, u128),
    clone_as_child: (u128, u128),
}
static mut bench_data: BenchData = BenchData {
    evaluator: (0, 0),
    check_conflict: (0, 0),
    apply_move: (0, 0),
    gen_moves: (0, 0),
    apply_key: (0, 0),
    solve_d0: (0, 0),
    clone_as_child: (0, 0),
};
pub fn bench_reset () {
    unsafe {
        bench_data = BenchData {
            evaluator: (0, 0),
            check_conflict: (0, 0),
            apply_move: (0, 0),
            gen_moves: (0, 0),
            apply_key: (0, 0),
            solve_d0: (0, 0),
            clone_as_child: (0, 0),
        };
    }
}
pub fn print_bench_result () { unsafe {

    let d = &bench_data;
    println!("=== Bench Result ==="); 
    {let t = d.evaluator; println!("evaluator: avg dt: {}, cnt: {}, total dt: {:.3}", t.0, t.1, (t.0*t.1) as f64/1000.0);}
    {let t = d.gen_moves; println!("gen_moves: avg dt: {}, cnt: {}, total dt: {:.3}", t.0, t.1, (t.0*t.1) as f64/1000.0);}
    {let t = d.apply_key; println!("apply_key: avg dt: {}, cnt: {}, total dt: {:.3}", t.0, t.1, (t.0*t.1) as f64/1000.0);}
    {let t = d.clone_as_child; println!("clone_as_child: avg dt: {}, cnt: {}, total dt: {:.3}", t.0, t.1, (t.0*t.1) as f64/1000.0);}
    {let t = d.solve_d0; println!("solve_d0: avg dt: {}, cnt: {}, total dt: {:.3}", t.0, t.1, (t.0*t.1) as f64/1000.0);}
}}