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

    pub fn clone_as_child (&self, mut field: Field, mov: &Move) -> State {
        let mut props: Props = Props { 
            sum_atk: self.props.sum_atk + self.props.atk,
            sum_ds: self.props.sum_ds + self.props.ds,
            sum_no_atk: self.props.sum_no_atk + 
                if self.props.ds > 0 && self.props.atk == 0 {self.props.ds} else {0},
            atk: 0,
            ds: 0,
            ..self.props
        };
        
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

use std::fmt;

impl fmt::Display for State {
    fn fmt (&self, f: &mut fmt::Formatter) -> fmt::Result { 
        for y in 0..20 {
            for x in 0..10 {
                let b: bool = (self.field.m[y] & (1 << x)) >> x == 1;
                if b {
                    write!(f, "# ")?;
                } else {
                    write!(f, ". ")?;
                }
            }
            print!(" ");
            match y {
                0 => write!(f, "b2b:   {:>2}", self.props.b2b)?,
                1 => write!(f, "combo: {:>2}", self.props.combo)?,
                3 => write!(f, "hold:  {:?}", self.hold)?,
                4 => write!(f, "queue:")?,
                5..=9 => if self.pieces.len() > y-5 {
                    write!(f, "{:?}", self.pieces[y-5])?
                },
                _ => ()
            };
            write!(f, "\n")?;
        }
        write!(f, "\n")?;
        Ok(())
    }
}