use crate::bench_data;
use std::time::Instant;

use super::{State, Field, Props};
use super::mac::*;

#[derive(Copy, Clone)]
pub enum EvaluatorMode {
    Norm,
    Attack,
    DS,
}
struct Consts {
    ds_height_threshold: f32,
    ds_mode_penalty: f32,
}
struct Factors {
    ideal_h: f32,
    well_threshold: f32,
}
struct Weights {
    hole: f32,
    hole_depth: f32,
    h_local_deviation: f32,
    h_global_deviation: f32,
    average_h: f32,
    sum_attack: f32, 
    sum_downstack: f32,
    attack: f32, 
    downstack: f32,
    eff: f32,
}
const WEIGHTS_ATK: Weights = Weights {
    hole: -100.0,
    hole_depth: -10.0,
    h_local_deviation: -5.0,
    h_global_deviation: -4.0,
    average_h :-10.0,
    sum_attack: 40.0,
    sum_downstack: 15.0,
    attack: 35.0,
    downstack: 10.0,
    eff: 50.0,
};

const WEIGHTS_DS: Weights = Weights {
    hole: -150.0,
    hole_depth: -20.0,
    h_local_deviation: -10.0,
    h_global_deviation: -8.0,
    average_h : -20.0,
    sum_attack: 0.0,
    sum_downstack: 35.0,
    attack: 0.0,
    downstack: 30.0,
    eff: 0.0,
};
const FACTORS_ATK: Factors = Factors {
    ideal_h: 5.0,
    well_threshold: 4.0,
};
const FACTORS_DS: Factors = Factors {
    ideal_h: 0.0,
    well_threshold: 20.0,
};
const CONSTS: Consts = Consts {
    ds_height_threshold: 14.0,
    ds_mode_penalty: -2000.0,
};

pub fn evaluate (state: &State, mode: EvaluatorMode) -> f32 {
    let start = Instant::now();
    defer!(unsafe {
        bench_data.evaluator.1 += 1;
        let dt = start.elapsed().as_micros();
        bench_data.evaluator.0 = if bench_data.evaluator.0 == 0 {dt} else {(bench_data.evaluator.0 + dt) / 2};
    });

    let f: &Field = &state.field;
    let p: &Props = &state.props;
    let mut score: f32 = 0.0;
    const FW: usize = 10;
    const FH: usize = 20;
    let mut h: [u8; FW] = [0; FW];
    let mut well: Option<u8> = None;

    // get all column heights
    {
        let mut cache_y: usize = 0;
        while cache_y < FH && f.m[cache_y] == 0 {
            cache_y += 1;
        }
        for x in 0..FW {
            for y in cache_y..=FH {
                if y == FH {
                    h[x] = 20;
                } else if f.m[y] & ( 1 << x ) > 0 {
                    h[x] = y as u8;
                    break;
                }
            }
        }
        dev_log!(ln, "h: {:?}", h);
    }
    // Get raw avg height 
    let mut avg: f32 = h.iter().sum::<u8>() as f32 / FW as f32;

    // find holes
    let (holes, hole_depth_sum_sq) = {
        let mut holes: f32 = 0.0;
        let mut depth_sum_sq: f32 = 0.0;
        for x in 0..FW {
            for y in (h[x] as usize + 1)..FH {
                if ( f.m[y] & ( 1 << x ) ) == 0 {
                    holes += 1.0;
                    let d: f32 = ((y - h[x] as usize) as f32).abs().min(3.0);
                    depth_sum_sq += d * d;
                }
            }
        }
        (holes, depth_sum_sq)
    };

    // Select weights
    // CURRENT SETTING: (for ds)
    // -> if holes
    // -> if average height past threshold
    let (weights, factors) = {
        match mode {
            EvaluatorMode::Norm => 
                if  FH as f32 - avg > CONSTS.ds_height_threshold || holes > 0.0 {
                    dev_log!(ln, "DS penalty: {}", CONSTS.ds_mode_penalty);
                    score += CONSTS.ds_mode_penalty;
                    (WEIGHTS_DS, FACTORS_DS)
                } else {
                    (WEIGHTS_ATK, FACTORS_ATK)
                },
            EvaluatorMode::DS => (WEIGHTS_DS, FACTORS_DS),
            EvaluatorMode::Attack => (WEIGHTS_ATK, FACTORS_ATK),
        }
    };

    // Score by holes & depth (split from calculation because weight selection requires hole info)
    {
        score += holes * weights.hole;
        score += hole_depth_sum_sq * weights.hole_depth;
        dev_log!(ln, "holes: {}, penalty: {}", holes, holes * weights.hole);
        dev_log!(ln, "hole depth sq sum: {}, penalty: {}", hole_depth_sum_sq, hole_depth_sum_sq * weights.hole_depth); 
    }
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
            avg = (avg * FW as f32 - h[well as usize] as f32) / (FW - 1) as f32;
        } 
        if let Some(x) = well {
            dev_log!(ln, "identified well: \x1b[1m{}\x1b[0m", x);
        }
    }

    // Score by avg height
    { 
        let h: f32 = FH as f32 - avg;
        let d: f32 = (h - factors.ideal_h).abs();
        score += weights.average_h * d * d;
        dev_log!(ln, "global h: {}, ideal: {}, penalty: {}", h, factors.ideal_h, d * d * weights.average_h); 
    }
    
    // Score by delta from average
    {
        dev_log!("global h-deviations: ");
        let mut sum_sq: f32 = 0.0;
        for x in 0..FW {
            if let Some(w) = well { // Ignore if well
                if w == x as u8 {
                    continue
                }
            }

            let d: f32 = avg - h[x] as f32;
            
            dev_log!("{} ", d);
            sum_sq += d * d;
        }
        dev_log!(ln, ", sum_sq: {}, penalty: {}", sum_sq, sum_sq * weights.h_global_deviation);
        score += sum_sq * weights.h_global_deviation;
    }
    // Local Height Deviation (from neighbor)
    {
        dev_log!("local h-deviation: ");
        let mut sum_sq: f32 = 0.0;
        let mut prev: Option<u8> = None;
        for x in 0..FW {
            if let Some(w) = well { // Ignore if well
                if w == x as u8 {
                    continue
                }
            }
            if let Some(prev) = prev { 
                let d: f32 = h[x].abs_diff(prev) as f32;
                sum_sq += d * d;
                dev_log!("{} ", d);
            } else {dev_log!("- ");}

            prev = Some(h[x]);
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
    }

    dev_log!(ln, "final score: \x1b[1m{}\x1b[0m", score);
    dev_log!("{}", state);

    return score;
}


#[cfg(test)] 
mod test {
    use super::*;   
    
    #[test]
    fn eval_test () {
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
            0b0_0_0_0_0_0_0_0_0_1,
            0b0_1_0_1_1_0_0_0_0_1,
            0b1_1_1_1_1_1_1_1_0_1,
            0b1_1_1_1_1_1_0_1_1_1,
        ];
        let mut state = State::new();
        
        state.field = field;
        state.props = Props::new();
        state.props.sum_atk = 0;
        state.props.sum_ds = 0;
        state.props.atk = 1;
        state.props.ds = 2;
        state.props.b2b = 0;
        state.props.combo = 0;
    
        dev_log!(ln, "score: \x1b[1m{}\x1b[0m", evaluate(&state, EvaluatorMode::Norm));  
    }
}