use std::collections::HashMap;
use super::{State, Field, Move, gen_moves, evaluate};

pub fn solve (state: &State, depth: u8) -> Option<(State, Move, f32)> {
    const CUTOFF_FACTOR: f32 = 0.25;

    let moves: HashMap<Field, Move> = gen_moves(state);
    let mut queue: Vec<(State, Move, f32)> = vec![];
    queue.reserve(moves.len());

    // Evaluate all children
    for (field, mov) in moves.iter() {
        let nstate: State = state.clone_as_child(field.clone(), mov);
        let score = evaluate(&nstate);
        queue.push((nstate, mov.clone(), score));
    }
    // Sort reverse
    queue.sort_by(|a, b| a.2.total_cmp(&b.2));

    // If no further expansion
    if depth == 0 {
        return queue.pop()
    }

    // Expand on top margin.
    let mut out: (State, Move, f32) = (State::new(), Move::new(), f32::NAN);
    let cutoff: usize = (CUTOFF_FACTOR * queue.len() as f32).floor() as usize;
    for _ in 0..cutoff {
        let (nstate, mov, score) = queue.pop().unwrap();

        if let Some(res) = solve(&nstate, depth-1) {
            if out.2.is_nan() || res.2 > out.2 {
                out = (nstate, mov, res.2);
            }
        }
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Piece;
    
    #[test]
    fn solve_test () {
        let mut state: State = State::new();
        state.pieces.push_back(Piece::Z);
        state.pieces.push_back(Piece::I);
        state.pieces.push_back(Piece::J);
        state.pieces.push_back(Piece::S);
        state.pieces.push_back(Piece::O);
        state.hold = Piece::L;

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
            0b0_0_0_0_0_0_1_1_1_0,
            0b0_0_0_1_1_1_1_1_1_1,
            0b0_0_1_1_1_1_1_1_1_1,
            0b0_0_1_1_1_1_1_1_1_1,
            0b0_1_1_1_1_1_1_1_1_1,
            0b0_1_1_1_1_1_1_1_1_1,
            0b0_1_1_1_1_1_1_1_1_1,
            0b0_1_1_1_1_1_1_1_1_1,
            0b0_1_1_1_1_1_1_1_1_1,
            0b0_1_1_1_1_1_1_1_1_1,
        ];
    
        if let Some(out) = solve(&state, 2) {
            // Log out result
            println!("result score: \x1b[1m{}\x1b[0m", out.2);
            println!("{}", &out.0);
        } else {
            println!("No results found.");
        }
    }
}