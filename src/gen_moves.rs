//! Module isolating `gen_moves` function

use std::collections::{VecDeque};

use ahash::{AHashMap, AHashSet};

use super::{Field, Move, State, Key, Piece};


 
/// Generates all valid Moves that can be applied to a given state. 
///
/// Implemented with BFS for finesse. 
/// Starting with the base move, expand it by adding another key to the move.
/// Append only valid and unique moves into the BFS queue. 
/// Uniqueness of Field is guarenteed via a Hashset<T>. This, in turn, guarentees uniqueness in Moves.
pub fn gen_moves(state: &State) -> AHashMap<Field, Move> {
    let _bencher: Option<crate::Bencher> = if cfg!(feature = "bench") {
        unsafe {
            Some( crate::Bencher::new( &mut crate::BENCH_DATA.gen_moves ) )
        }
    } else {None};

    // Check if there is even a piece to expand on.
    if state.pieces.is_empty() {
        return AHashMap::new();
    }
    let piece: &Piece = &state.pieces[0];
    let hold : &Piece = if state.hold == Piece::None { &state.pieces[1] } else { &state.hold };

    let mut field_hash: AHashMap<Field, Move> = AHashMap::new();
    let mut move_hash: AHashSet<u64> = AHashSet::new();
    let mut q: VecDeque<Move> = VecDeque::new();
    q.reserve(64);

    // Base cases for BFS
    {
        for r in [Key::Cw, Key::Ccw, Key::_180] {
            for x in 0..10 {
                let mut m = Move {
                    hold: false,
                    x,
                    y: 1,
                    r: 0,
                    s: -1,
                    tspin: false,
                    lock: false,
                    list: 0,
                }; 
                for _ in 0..(4-x) {
                    m.add_to_list(&Key::Left);
                }
                for _ in 0..(x-4) {
                    m.add_to_list(&Key::Right);
                }
                let mut h = m.clone();
                h.hold = true;

                m.apply_key(&r, &state.field, piece, hold);
                m.apply_key(&Key::SoftDrop, &state.field, piece, hold);

                h.apply_key(&r, &state.field, piece, hold);
                h.apply_key(&Key::SoftDrop, &state.field, piece, hold);

                if !state.field.check_conflict(&m, piece) {
                    move_hash.insert(m.hash());
                    q.push_back(m); 
                }
                if !state.field.check_conflict(&h, piece) {
                    move_hash.insert(h.hash());
                    q.push_back(h); 
                }
            }
        }
    }

    while let Some(mov) = q.pop_front() {
        for key in [Key::Left, Key::Right, Key::Cw, Key::Ccw, Key::_180, Key::SoftDrop, Key::HardDrop] {
            let mut m = mov.clone();
            if !m.apply_key(&key, &state.field, piece, hold) {
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
//        state.pieces.push_back(Piece::S);
        state.pieces.push_back(Piece::T);
        state.pieces.push_back(Piece::I);
        state.pieces.push_back(Piece::J);
        state.pieces.push_back(Piece::L);
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
            0b0_0_0_0_0_0_1_0_0_0,
            0b0_0_0_0_0_1_1_0_0_1,
            0b1_1_1_1_1_1_0_0_0_1,
            0b1_1_1_1_1_1_1_0_1_1,
        ];

    
        let map = gen_moves(&state);
        for (field, _) in map {
            println!("{}", field);
        }
    }
}
