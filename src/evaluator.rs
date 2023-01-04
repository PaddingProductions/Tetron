use super::{Clear, State, Field, Props};

macro_rules! dev_log {
    ($s:literal) => {
        if cfg!(test) {
            print!($s);
        }
    };
    (ln, $s:literal) => {
        if cfg!(test) {
            println!($s);
        }
    };
    ($s:literal, $($a: expr),* ) => {
        if cfg!(test) {
            print!(
                $s,
                $($a,)*
            );
        }
    };
    (ln, $s:literal, $($a: expr),* ) => {
        if cfg!(test) {
            println!(
                $s,
                $($a,)*
            );
        }
    };
}


struct Weights {
    hole: f32,
    h_local_deviation: f32,
    h_global_deviation: f32,
    average_h: f32,
    attack: f32, 
    no_attack_clear: f32,
}
impl Weights {
    pub fn norm () -> Self {
        Self {
            hole: -50.0,
            h_local_deviation: -5.0,
            h_global_deviation: -10.0,
            average_h :-10.0,
            attack: 30.0,
            no_attack_clear: -35.0,
        }
    }
}
pub fn evaluate (state: &State) -> f32 {
    let f: &Field = &state.field;
    let p: &Props = &state.props;
    let mut score: f32 = 0.0;
    const f_w: usize = 10;
    const f_h: usize = 20;
    let mut h: [u8; f_w] = [0; f_w];
    
    // Get weights
    let weights = Weights::norm();

    // get all column heights
    {
        let mut cache_y: usize = 0;
        while cache_y < f_h && f.m[cache_y] == 0 {
            cache_y += 1;
        }
        for x in 0..f_w {
            for y in cache_y..=f_h {
                if y == f_h {
                    h[x] = 20;
                } else if f.m[y] & ( 1 << x ) > 0 {
                    h[x] = y as u8;
                    break;
                }
            }
        }
        dev_log!(ln, "h: {:?}", h);
    }
    
    // find holes
    {
        let mut hole_score: f32 = 0.0;
        for x in 0..f_w {
            for y in (h[x] as usize + 1)..f_h {
                if ( f.m[y] & ( 1 << x ) ) == 0 {
                    hole_score += weights.hole;
                }
            }
        }
        score += hole_score;
        dev_log!(ln, "holes: {}, penalty: {}", hole_score / weights.hole, hole_score);
    }
    // Local Height Deviation (from neighbor)
    {
        dev_log!("local h-deviation: ");
        let mut sum_d: u8 = 0;
        for x in 1..f_w {
            let d: u8 = h[x].abs_diff(h[x-1]);
            sum_d += d;
            dev_log!("{} ", d);
        }
        score += sum_d as f32 * weights.h_local_deviation;
        dev_log!(ln, ", sum: {}, penalty: {}", sum_d, sum_d as f32* weights.h_local_deviation);
    }
    // Global Height Deviation (from avg)
    // NOTE: different from misamino, it processes with all values * f_w to avoid floating point arithmetic. 
    {
        let mut sum_sq: f32 = 0.0;
        let avg: f32 = h.iter().sum::<u8>() as f32 / f_w as f32;
        { // Score by avg height
            let d: f32 = f_h as f32 - avg;
            score += weights.average_h * d * d;
            dev_log!(ln, "global h: {}, penalty: {}", d, d * d * weights.average_h); 
        }
    
        dev_log!("global h-deviations: ");
        // Score by delta from average
        for x in 0..f_w {
            const H_DELTA_CAP: f32 = 5.0;
            let d: f32 = (avg - h[x] as f32).max(-H_DELTA_CAP).min(H_DELTA_CAP);
            
            dev_log!("{} ", d);
            sum_sq += d * d;
        }
        dev_log!(ln, ", sum_sq: {}, penalty: {}", sum_sq, sum_sq * weights.h_global_deviation / 1000.0);
        score += sum_sq * weights.h_global_deviation / 1000.0;
    }
    // clear and attack
    {
        let att = match p.clear {
            Clear::None => 0,
            Clear::Clear1 => 0,
            Clear::Clear2 => 1,
            Clear::Clear3 => 2,
            Clear::Clear4 => 4,
            Clear::Tspin1 => 2,
            Clear::Tspin2 => 4,
            Clear::Tspin3 => 6,
        };
        let clear = match p.clear {
            Clear::None => 0,
            Clear::Clear1 => 1,
            Clear::Clear2 => 2,
            Clear::Clear3 => 3,
            Clear::Clear4 => 4,
            Clear::Tspin1 => 1,
            Clear::Tspin2 => 2,
            Clear::Tspin3 => 3,
        };
        score += att as f32 * weights.attack;
            
        if att == 0 {
            score += clear as f32 * weights.no_attack_clear;
        }
    }
    return score;
}

pub fn eval_test () {
    let mut field = Field::new();
    field.m = [
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
        0b0_0_0_0_0_0_0_0_0_0,
        0b0_0_0_0_0_1_1_1_1_0,
    ];
    let mut state = State::new();
    state.field = field;
    state.props = Props::new();
    state.props.sum_atk = 0;
    state.props.sum_ds = 0;
    state.props.b2b = 0;
    state.props.combo = 0;
    //state.props.clear = Clear::Clear4;

    dev_log!(ln, "score: \x1b[46;1m{}\x1b[0m", evaluate(&state));        
}

#[cfg(test)] 
mod test {
    use super::*;    
}