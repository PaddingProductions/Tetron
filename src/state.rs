use std::collections::VecDeque;
use super::{Props, Field, Piece, Move};

pub struct State {
    pub pieces: VecDeque<Piece>,
    pub hold: Piece,
    pub field: Field,
    pub props: Props,
}
impl State {
    pub fn new () -> Self {
        let mut pieces = VecDeque::new();
        pieces.reserve(6);

        Self {
            pieces,
            hold: Piece::None,
            field: Field::new(),
            props: Props::new()
        }
    }

    pub fn clone_as_child (self: &Self, mut field: Field, mov: &Move) -> State {
        let mut props: Props = self.props.clone();
        
        // process field and edit properties.
        field.set_props(mov, &mut props);

        // generate children's piece queue and hold piece.
        let mut pieces: VecDeque<Piece> = self.pieces.clone();
        let mut hold: Piece = self.hold;
        if mov.hold {
            hold = pieces.pop_front().unwrap();
            if self.hold == Piece::None {
                pieces.pop_front();
            }
        } else {
            pieces.pop_front();
        }

        Self {
            field,
            pieces,
            hold,
            props,
        }
    }
}