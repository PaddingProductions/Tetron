//! Module isolating `solve()` function.

use super::{State, Field, Move, gen_moves, evaluate};
use super::config::Config;

use std::collections::HashMap;
use rayon::prelude::*;

const INHERITANCE_F: f32 = 0.0;
const SCORE_CUTOFF_FACTOR: [f32; 3] = [0.4, 0.3, 0.25];
const STRICT_CUTOFF: [usize; 3] = [12, 11, 10];
static mut EXPANSIONS: u32 = 0;

/// Core function of Tetron. Produces an optimal move from input state.
///
/// `depth` parameter configures DFS depth in exploration.
/// `mode` parameter alters bot behavior & priority. Defaults to `Norm`. Used for topical testing. 
/// 
/// Returns the selected Move, the resultant State, and the calculated score.
/// Bot behavior configurable via source code. 
pub fn solve (state: &State, configs: &Config) -> Option<(State, Move, f32)> {
    // Benching
    let _bencher: Option<crate::Bencher> = if cfg!(feature = "bench") {
        unsafe {
            Some( crate::Bencher::new( &mut crate::BENCH_DATA.solve_d0 ) )
        }
    } else {None};   

    let moves: HashMap<Field, Move> = gen_moves(state);
    let mut queue: Vec<(State, Move, f32)> = vec![];
    queue.reserve(moves.len());

    // Evaluate all children
    for (field, mov) in moves.iter() {
        let nstate: State = state.clone_as_child(field.clone(), mov);
        let score = evaluate(&nstate, configs.eval_mode);
        queue.push((nstate, mov.clone(), score));
    }
    // Sort reverse
    queue.sort_by(|a, b| a.2.total_cmp(&b.2));


    // If empty (game over)
    if queue.is_empty() {
        return None;
    }

    // If no further expansion
    if configs.depth == 0 {
        return queue.pop()
    }

    // Process Cutoff, dropping others.
    {
        let score_variation = queue[queue.len()-1].2 - queue[0].2;
        let cutoff_score: f32 = queue[queue.len()-1].2 - (score_variation * SCORE_CUTOFF_FACTOR[configs.depth as usize - 1]);
        let mut cutoff: usize = 0;
        for i in 0..queue.len() {
            if queue[i].2 > cutoff_score {
                cutoff = i;
                break;
            }
        }
        cutoff = cutoff.max(queue.len() - queue.len().min(STRICT_CUTOFF[configs.depth as usize - 1] - 1));
        queue.drain(0..(cutoff-1));
    }
    
    // Expand & Sort
    let next_configs = configs.next();
    queue.par_iter_mut()
    //queue.iter_mut()
        .for_each(|(nstate, _, score)| 
            if let Some(res) = solve(&nstate, &next_configs) {
                let nscore: f32 = *score * INHERITANCE_F + res.2 * (1.0 - INHERITANCE_F);
                *score = nscore;
            } else {
                *score = f32::NEG_INFINITY;
            }
        );
    queue.sort_by(|a, b| a.2.total_cmp(&b.2));
    
    //println!("expanded: {} @ depth={}", queue.len(), depth);
    unsafe {
        EXPANSIONS += queue.len() as u32;
        if configs.depth == 3 {
            println!("expansions: {}", EXPANSIONS); 
            EXPANSIONS = 0;
        }
    }
    queue.pop()
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;
    use crate::{Piece, bench_increment_solve};

    #[test]
    fn solve_test () {
        crate::bench_reset();

        let mut state: State = State::new();
        state.pieces.push_back(Piece::T);
        state.pieces.push_back(Piece::O);
        state.pieces.push_back(Piece::S);
        state.pieces.push_back(Piece::Z);
        state.pieces.push_back(Piece::S);
        state.hold = Piece::J;

        state.field.m = [   
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
            0b0_0_0_0_0_0_0_1_0_0,
            0b0_0_0_0_1_0_0_1_1_0,
            0b1_1_1_1_1_0_0_0_1_1,
            0b1_1_1_1_1_1_0_1_1_1,
        ];

        bench_increment_solve();
        let start = if cfg!(feature = "bench") { Some(Instant::now()) } else { None };

        if let Some(out) = solve(&state, &Config::new(0, crate::evaluator::EvaluatorMode::Norm)) {
            
            // Log out result
            println!("result score: \x1b[1m{}\x1b[0m", out.2);
            println!("{}", &out.0);
            println!("move: {:?}", &out.1);
            println!("keys: {:?}", &out.1.parse_list());
            println!("prop: {:?}", &out.0.props);

            // Time
            if let Some(time) = start {
                let dt = time.elapsed().as_millis();
                println!("dt: \x1b[1m{}\x1b[0mms", dt);
            }
        } else {
            println!("No results found.");
        }
        crate::print_bench_result();
    }
}
