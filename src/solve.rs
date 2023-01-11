use std::collections::HashMap;
use super::{State, Field, Move, gen_moves, evaluate, EvaluatorMode};

const INHERITANCE_F: f32 = 0.0;
const SCORE_CUTOFF_FACTOR: f32 = 0.3;
const STRICT_CUTOFF: usize = 8;

pub fn solve (state: &State, depth: u8, mode: Option<EvaluatorMode>) -> Option<(State, Move, f32)> {
    let mode = mode.unwrap_or_else(|| EvaluatorMode::Norm);
    let moves: HashMap<Field, Move> = gen_moves(state);
    let mut queue: Vec<(State, Move, f32)> = vec![];
    queue.reserve(moves.len());

    // Evaluate all children
    for (field, mov) in moves.iter() {
        let nstate: State = state.clone_as_child(field.clone(), mov);
        let score = evaluate(&nstate, mode);
        queue.push((nstate, mov.clone(), score));
    }
    // Sort reverse
    queue.sort_by(|a, b| a.2.total_cmp(&b.2));

    // If empty (game over)
    if queue.is_empty() {
        return None;
    }

    // If no further expansion
    if depth == 0 {
        return queue.pop()
    }

    // Expand on top margin.
    let mut out: (State, Move, f32) = (State::new(), Move::new(), f32::NAN);
    let score_variation = queue[queue.len()-1].2 - queue[0].2;
    let cutoff_score: f32 = queue[queue.len()-1].2 - (score_variation * SCORE_CUTOFF_FACTOR);
    let mut cnt = 0;
     
    while let Some((nstate, mov, score)) = queue.pop() {
        if cnt >= STRICT_CUTOFF {
            break;
        }
        if score < cutoff_score {
            break;
        }

        if let Some(res) = solve(&nstate, depth-1, Some(mode)) {
            let nscore = score * INHERITANCE_F + res.2 * (1.0 - INHERITANCE_F);
            if out.2.is_nan() || nscore > out.2 {
                out = (nstate, mov, nscore);
            }
        }
        cnt += 1;
    }
    //println!("solve expands @depth_{}: {}", depth, cnt);
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Piece;
    
    #[test]
    fn solve_test () {
        let mut state: State = State::new();
        state.pieces.push_back(Piece::T);
        state.pieces.push_back(Piece::J);
        state.pieces.push_back(Piece::I);
        state.pieces.push_back(Piece::S);
        state.pieces.push_back(Piece::Z);
        state.hold = Piece::T;

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
            0b0_0_0_0_0_0_0_0_0_0,
            0b0_0_0_0_0_0_0_0_1_1,
            0b1_1_1_1_1_1_0_0_0_1,
            0b1_1_1_1_1_1_1_0_1_1,
        ];
    
        if let Some(out) = solve(&state, 2, None) {
            // Log out result
            println!("result score: \x1b[1m{}\x1b[0m", out.2);
            println!("{}", &out.0);
            println!("move: {:?}", &out.1);
            println!("prop: {:?}", &out.0.props);
        } else {
            println!("No results found.");
        }
    }
}