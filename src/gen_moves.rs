//! Module isolating `gen_moves` function

use std::collections::{HashMap, HashSet, VecDeque};

use super::{Field, Move, State, Key, Piece};


 
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

    // Precompute 
    let conflict_cache = (state.field.precompute_conflict(piece), state.field.precompute_conflict(hold));

    // Base cases for BFS
    let starting_move: Move = Move::new();
    let mut starting_move_hold = starting_move.clone();
    starting_move_hold.apply_key(&Key::Hold, conflict_cache, &state.field, piece, hold);


    // Check if field does not allow base moves (game over by top-out)
    if !Field::check_conflict(conflict_cache.0, &starting_move) {
        q.push_back(starting_move);
    }
    if !Field::check_conflict(conflict_cache.1, &starting_move_hold) {
        q.push_back(starting_move_hold);
    }

    while !q.is_empty() {
        let mov: Move = q.pop_front().unwrap();

        for key in [Key::Left, Key::Right, Key::Cw, Key::Ccw, Key::_180, Key::SoftDrop, Key::HardDrop] {
            let mut m = mov.clone();
            if !m.apply_key(&key, conflict_cache, &state.field, piece, hold) {
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
