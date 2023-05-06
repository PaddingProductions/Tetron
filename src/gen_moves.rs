//! Module isolating `gen_moves` function

use std::collections::{HashMap, HashSet, VecDeque};

use super::{Field, Move, State, Key, Piece};
use crate::field::ConflictCache;

 
/// Generates all valid Moves that can be applied to a given state. 
///
/// Implemented with BFS for finesse. 
/// Starting with the base move, expand it by adding another key to the move.
/// Append only valid and unique moves into the BFS queue. 
/// Uniqueness of Field is guarenteed via a Hashset<T>. This, in turn, guarentees uniqueness in Moves.
pub fn gen_moves(state: &State) -> HashMap<Field, Move> {
    let _bencher: Option<crate::Bencher> = if cfg!(feature = "bench") {
        unsafe {
            Some( crate::Bencher::new( &mut crate::BENCH_DATA.gen_moves ) )
        }
    } else {None};

    // Check if there is even a piece to expand on.
    if state.pieces.is_empty() {
        return HashMap::new();
    }
    let piece: &Piece = &state.pieces[0];
    let hold: &Piece = if state.hold == Piece::None { &state.pieces[1] } else { &state.hold };

    let mut field_hash: HashMap<Field, Move> = HashMap::new();
    let mut move_hash: HashSet<u64> = HashSet::new();
    let mut q: VecDeque<Move> = VecDeque::new();
    q.reserve(40);

    let mut cache: (ConflictCache, ConflictCache) = ([[0; 20]; 4], [[0; 20]; 4]);

    // Base cases for BFS
    {
        let m: Move = Move::new();
        if !state.field.check_conflict(&mut cache.0, &m, piece) {
            q.push_back(m);
        }
    } 
    // Hold base case
    if *hold != Piece::None {
        let mut m = Move::new();
        m.apply_key(&Key::Hold, &mut cache, &state.field, piece, hold);
        if !state.field.check_conflict(&mut cache.1, &m, piece) {
            q.push_back(m);
        }
    }

    while !q.is_empty() {
        let mov: Move = q.pop_front().unwrap();

        for key in [Key::Left, Key::Right, Key::Cw, Key::Ccw, Key::_180, Key::SoftDrop, Key::HardDrop] {
            let mut m = mov.clone();
            if !m.apply_key(&key, &mut cache, &state.field, piece, hold) {
                continue;
            }

            // Check Move hash
            if move_hash.get(&m.hash()).is_some() {
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
                move_hash.insert(m.hash());
                if m.list_len() < 10 {
                    q.push_back(m);
                }
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
        //state.pieces.push_back(Piece::Z);
        //state.pieces.push_back(Piece::I);
        //state.pieces.push_back(Piece::J);
        state.pieces.push_back(Piece::L);
        state.pieces.push_back(Piece::S);
        state.hold = Piece::Z;

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
            0b0_0_0_0_0_0_0_0_0_0,
            0b1_1_1_1_1_0_0_0_0_0,
            0b1_1_1_1_1_1_1_0_0_0,
        ];

    
        let map = gen_moves(&state);
        for (field, _) in map {
            println!("{}", field);
        }
    }
}
