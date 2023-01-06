use super::{State, Field, Props};
use super::mac::*;

struct Factors {
    ideal_h: f32,
    well_threshold: f32,
}
impl Factors {
    pub fn norm () -> Self {
        Self {
            ideal_h: 8.0,
            well_threshold: 4.0,
        }
    }
}
struct Weights {
    hole: f32,
    h_local_deviation: f32,
    h_global_deviation: f32,
    average_h: f32,
    sum_attack: f32, 
    sum_downstack: f32,
    attack: f32, 
    downstack: f32,
    no_attack_clear: f32,
    eff: f32,
}
impl Weights {
    pub fn norm () -> Self {
        Self {
            hole: -150.0,
            h_local_deviation: -5.0,
            h_global_deviation: -10.0,
            average_h :-10.0,
            sum_attack: 30.0,
            sum_downstack: 10.0,
            attack: 35.0,
            downstack: 10.0,
            no_attack_clear: 0.0,
            eff: 50.0,
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
    let mut well: Option<u8> = None;

    // Get weights
    let weights = Weights::norm();
    let factors = Factors::norm();

    dev_log!("{}", state);

    
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
    
    let mut avg: f32 = h.iter().sum::<u8>() as f32 / f_w as f32;
    // Find well (max neg deviation from avg > than threshold)
    {
        for x in 0..10 {
            let d: f32 = avg - h[x] as f32;
            if d < 0.0 && d.abs() >= factors.well_threshold {
                if let Some(pwell) = well {
                    if avg - h[pwell as usize] as f32 > d {
                        well = Some(x as u8);    
                    }
                } else {
                    well = Some(x as u8);
                }
            }
        }
        if let Some(well) = well {
            avg = (avg * f_w as f32 - h[well as usize] as f32) / (f_w - 1) as f32;
        } 
        if let Some(x) = well {
            dev_log!(ln, "identified well: \x1b[1m{}\x1b[0m", x);
        }
    }

    // Score by avg height
    { 
        let h: f32 = f_h as f32 - avg;
        let d: f32 = (h - factors.ideal_h).abs();
        score += weights.average_h * d * d;
        dev_log!(ln, "global h: {}, ideal: {}, penalty: {}", h, factors.ideal_h, d * d * weights.average_h); 
    }
    
    // Score by delta from average
    {
        dev_log!("global h-deviations: ");
        let mut sum_sq: f32 = 0.0;
        for x in 0..f_w {
            if let Some(w) = well { // Ignore if well
                if w == x as u8 {
                    continue
                }
            }

            const H_DELTA_CAP: f32 = 5.0;
            let d: f32 = (avg - h[x] as f32).max(-H_DELTA_CAP).min(H_DELTA_CAP);
            
            dev_log!("{} ", d);
            sum_sq += d * d;
        }
        dev_log!(ln, ", sum_sq: {}, penalty: {}", sum_sq, sum_sq * weights.h_global_deviation / 1000.0);
        score += sum_sq * weights.h_global_deviation / 1000.0;
    }
    // Local Height Deviation (from neighbor)
    {
        dev_log!("local h-deviation: ");
        let mut sum_sq: f32 = 0.0;
        for x in 1..f_w {
            if let Some(w) = well { // Ignore if well
                if w == x as u8 {
                    continue
                }
            }
            
            let d: u8 = h[x].abs_diff(h[x-1]);
            sum_sq += (d * d) as f32;
            dev_log!("{} ", d);
        }

        score += sum_sq * weights.h_local_deviation;
        dev_log!(ln, ", sum: {}, penalty: {}", sum_sq, sum_sq * weights.h_local_deviation);
    }

    // clear and attack
    {
        dev_log!(ln, "sum_atk: {}, sum_ds: {}", p.sum_atk, p.sum_ds);
        score += (p.sum_atk as i8 - p.sum_ds as i8) as f32 * weights.eff;

        score += p.sum_atk as f32 * weights.sum_attack;
        score += p.sum_ds as f32 * weights.sum_downstack;
        score += p.atk as f32 * weights.attack;
        score += p.ds as f32 * weights.downstack;
        //score += p.sum_no_atk as f32 * weights.no_attack_clear;
        
    }
    dev_log!(ln, "final score: \x1b[1m{}\x1b[0m", score);
    return score;
}

pub fn eval_sandbox () {
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
        0b0_0_0_0_0_0_0_0_1_0,
        0b0_0_0_0_1_1_1_1_1_1,
        0b0_0_0_0_1_1_1_1_1_1,
        0b0_1_1_0_1_1_1_1_1_1,
        0b0_1_1_1_1_1_1_1_1_1,
        0b0_1_1_1_1_1_1_1_1_1,
        0b0_1_1_1_1_1_1_1_1_1,
    ];/*
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
        0b0_0_0_0_0_0_0_0_1_0,
        0b0_0_0_0_1_1_1_1_1_1,
        0b0_0_0_0_1_1_1_1_1_1,
        0b0_0_0_0_1_1_1_1_1_1,
        0b0_0_1_1_1_1_1_1_1_1,
        0b0_0_1_1_1_1_1_1_1_1,
        0b0_1_1_1_1_1_1_1_1_1,
    ]; */
    let mut state = State::new();
    state.field = field;
    state.props = Props::new();
    state.props.sum_atk = 0;
    state.props.sum_ds = 0;
    state.props.b2b = 0;
    state.props.combo = 0;
    //state.props.clear = Clear::Clear4;

    dev_log!(ln, "score: \x1b[1m{}\x1b[0m", evaluate(&state));        
}

#[cfg(test)] 
mod test {
    use super::*;   
    
    #[test]
    fn eval_test () {
        eval_sandbox();
    }
}