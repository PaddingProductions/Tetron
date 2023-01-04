use std::collections::HashMap;
use super::{State, Field, Move, gen_moves, evaluate};

pub fn solve (state: &State, depth: u8) -> (State, Move, f32) {
    
    let moves: HashMap<Field, Move> = gen_moves(state);
    let mut out: (State, Move, f32) = (State::new(), Move::new(), f32::NAN);

    for (field, mov) in moves.iter() {
        let nstate: State = state.clone_as_child(field.clone(), mov);
        
        if depth > 0 {
            let res = solve(&nstate, depth-1);
            if out.2.is_nan() || res.2 > out.2 {
                out = (nstate, mov.clone(), res.2);
            }
        } else {
            let score: f32 = evaluate(&nstate);
            if out.2.is_nan() || score > out.2 {
                out = (nstate, mov.clone(), score);
            }
        }
    }
    out
}

