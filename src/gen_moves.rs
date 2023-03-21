use std::collections::{HashMap, HashSet, VecDeque};
use std::time::Instant;

use super::{Field, Move, State, Key, Piece};
use crate::BENCH_DATA;


/* 
    Generates all valid Moves that can be applied to a given state. 
    Implemented via BFS for finesse. 
    Starting with the base move, expand it by adding another key to the move.
    Append only valid and unique moves into the BFS queue. 
    Uniqueness of Field is guarenteed via a Hashset<T>. This, in turn, guarentees uniqueness in Moves.
 */
pub fn gen_moves(state: &State) -> HashMap<Field, Move> {
    // Benching
    if cfg!(feature = "bench") {
        let start = Instant::now();
        defer!(unsafe { 
            BENCH_DATA.gen_moves.1 += 1;
            let dt = start.elapsed().as_micros();
            BENCH_DATA.gen_moves.0 = if BENCH_DATA.gen_moves.0 == 0 {dt} else {(BENCH_DATA.gen_moves.0 + dt) / 2};
        });
    }


    // Check if there is even a piece to expand on.
    if state.pieces.is_empty() {
        return HashMap::new();
    }
    let piece: &Piece = &state.pieces[0];
    let hold: &Piece = if state.hold == Piece::None { &state.pieces[1] } else { &state.hold };

    let mut field_hash: HashMap<Field, Move> = HashMap::new();
    let mut move_hash: HashSet<Move> = HashSet::new();
    let mut q: VecDeque<Move> = VecDeque::new();
    q.reserve(40);

    // Base cases for BFS
    let starting_move: Move = Move::new();
    let mut starting_move_hold = starting_move.clone();
    starting_move_hold.hold = true;


    // Check if field does not allow base moves (game over by top-out)
    if !state.field.check_conflict(&starting_move, piece) {
        q.push_back(starting_move);
    }
    if !state.field.check_conflict(&starting_move_hold, piece) {
        q.push_back(starting_move_hold);
    }

    while !q.is_empty() {
        let mov: Move = q.pop_front().unwrap();

        for key in [Key::Left, Key::Right, Key::Cw, Key::Ccw, Key::_180, Key::SoftDrop, Key::HardDrop] {
            let mut m = mov.clone();
            if !m.apply_key(&key, &state.field, piece, hold) {
                continue;
            }

            // Check Move hash
            if move_hash.get(&m).is_some() {
                continue;
            }
            // If harddropped, check field hash.
            if m.lock {
                if let Ok(field) = state.field.apply_move(&m, piece, hold) {
                    if !field_hash.contains_key(&field) {
                        field_hash.insert(field, m);
                    }
                }
            } else {
                move_hash.insert(m.clone());
                q.push_back(m);
            }
        }
    }
    field_hash
} 


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gen_moves_test () {
        let mut state: State = State::new();
        state.pieces.push_back(Piece::S);
        state.pieces.push_back(Piece::J);
        state.pieces.push_back(Piece::S);
        state.pieces.push_back(Piece::J);
        state.pieces.push_back(Piece::I);
        state.hold = Piece::I;
        
        state.field.m = [   
            0b0_0_0_0_0_0_0_0_0_0,
            0b0_0_0_0_0_0_0_0_0_0,
            0b0_0_0_0_0_0_0_0_0_0,
            0b0_0_0_0_0_0_0_0_0_0,
            0b0_0_0_0_0_0_1_1_1_0,
            0b0_0_0_0_0_0_1_1_1_1,
            0b0_0_0_0_0_0_1_1_1_1,
            0b0_0_0_0_0_0_1_1_1_1,
            0b0_0_0_0_0_0_1_1_1_1,
            0b0_0_0_0_0_0_1_1_1_1,
            0b0_0_0_0_0_0_1_1_1_1,
            0b0_0_0_0_0_0_1_1_1_1,
            0b1_0_1_1_0_0_1_1_1_0,
            0b1_1_1_1_1_0_1_1_1_1,
            0b1_1_1_1_1_0_1_1_1_1,
            0b1_1_1_1_1_0_1_1_1_1,
            0b1_1_1_1_1_0_1_1_1_1,
            0b1_1_1_1_1_0_1_1_1_1,
            0b1_1_1_1_1_0_1_1_1_1,
            0b1_1_1_1_1_0_1_1_1_1,
        ];
    
        let map = gen_moves(&state);
        for (field, _) in map {
            println!("{}", field);
        }
    }
}
