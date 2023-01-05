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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Piece;
    
    #[test]
    fn solve_test () {
        let mut state: State = State::new();
        state.pieces.push_back(Piece::J);
        state.pieces.push_back(Piece::L);
        state.pieces.push_back(Piece::Z);
        state.pieces.push_back(Piece::I);
        state.pieces.push_back(Piece::S);
        state.hold = Piece::I;

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
            0b0_0_0_1_1_0_0_0_0_1,
            0b0_0_1_1_1_1_0_0_1_1,
            0b0_1_1_1_1_1_0_1_1_1,
            0b0_1_1_1_1_1_0_1_1_1,
            0b0_1_1_1_1_1_1_1_1_1,
            0b0_1_1_1_1_1_1_1_1_1,
            0b0_1_1_1_1_1_1_1_1_1,
            0b0_1_1_1_1_1_1_1_1_1,
            0b1_1_1_1_1_1_0_1_1_1,
        ];
    
        let out: (State, Move, f32) = solve(&state, 1);

        // Log out result
        println!("result score: \x1b[1m{}\x1b[0m", out.2);
        println!("{}", &out.0);
    }
}