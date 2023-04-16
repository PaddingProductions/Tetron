use crate::evaluator::EvaluatorMode;



#[derive(Clone)]
pub struct Config {
    pub depth: u8,
    pub eval_mode: EvaluatorMode,
}

impl Config {
    pub fn new (depth: u8, eval_mode: EvaluatorMode) -> Self {
        Self {
            depth,
            eval_mode,
        }
    }
    pub fn next (&self) -> Self {
        Self {
            depth: self.depth-1,
            ..self.clone()
        }
    }
}
