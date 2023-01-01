use std::collections::HashMap;
use super::*;

/* 
    Generates all valid Moves that can be applied to a given state. 
    Implemented via BFS for finesse. 
    Starting with the base move, expand it by adding another key to the move.
    Append only valid and unique moves into the BFS queue. 
    Uniqueness of Field is guarenteed via a Hashset<T>. This, in turn, guarentees uniqueness in Moves.
 */
pub fn gen_moves(state: &State) -> HashMap<Field, Move> {
    // Check if there is even a piece to expand on.
    if state.pieces.is_empty() {
        return HashMap::new();
    }

    let mut hash: HashMap<Field, Move> = HashMap::new();

    let piece: &Piece = &state.pieces[0];
    let hold: &Piece = if state.hold == Piece::None { &state.pieces[1] } else { &state.hold };

    // Select hold & rotation.
    for h in [false, true] {
        for r in 0..3 {
            let mut m: Move = Move::new();
            if h { m.apply_key(&Key::Hold, &state.field, piece, hold); }
            let _ =  match r {
                1 => m.apply_key(&Key::Cw, &state.field, piece, hold),
                2 => m.apply_key(&Key::_180, &state.field, piece, hold),
                3 => m.apply_key(&Key::Ccw, &state.field, piece, hold),
                _ => true,
            };

            // Select direction, move till it no longer changes the placement.
            for d in [Key::Left, Key::Right] {
                loop {
                    if m.apply_key(&d, &state.field, piece, hold) {
                        // Add to map if generates unique field.
                        m.apply_key(&Key::HardDrop, &state.field, piece, hold);
                        println!("{:?}", m);
                        let field = state.field.apply_move(&m, piece, hold);
                        if !hash.contains_key(&field) {
                            hash.insert(field, m.clone());
                        }
                        m.y = 1;
                    } else {
                        break;
                    }
                }
            }
        }
    }
    hash
}