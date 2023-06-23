//! Module isolating heuristics function.


use super::{State, Field, Props, Piece};
use crate::mac::*;

/// Enumeration representing possible modes for heuristic function
#[derive(Copy, Clone, Debug)]
pub enum EvaluatorMode {
    /// Normal Mode - Evaluator decides mode
    Norm,
    /// Attack Mode - Prioritizes attacks.
    Attack,
    /// Downstack Mode - Prioritizes downstack.
    DS,
}
struct Consts {
    ds_height_threshold: f32,
    ds_hole_threshold: f32,
    ds_mode_penalty: f32,
    well_placement_f: f32,
    well_placement: [f32; 10],
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
    well_v: f32,
    well_parity: f32,
    well_odd_par: f32,
    well_flat_parity: f32, 
    tspin_flat_bonus: f32,
    tspin_dist: f32,
    tspin_completeness: f32,
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
    well_v: 1.0,
    well_parity: -3.0,
    well_odd_par: -30.0,
    well_flat_parity: 40.0, 
    tspin_flat_bonus: 40.0,
    tspin_dist: -0.0,
    tspin_completeness: 0.0,
    average_h : -1.0,
    sum_attack: 25.0,
    sum_downstack: 15.0,
    attack: 20.0,
    downstack: 10.0,
    eff: 100.0,
};

const WEIGHTS_DS: Weights = Weights {
    hole: -150.0,
    hole_depth: -20.0,
    h_local_deviation: -10.0,
    h_global_deviation: -8.0,
    well_v: 0.0,
    well_parity: 0.0,
    well_odd_par: 0.0,
    well_flat_parity: 0.0, 
    tspin_flat_bonus: -150.0, // Same as hole
    tspin_dist: 0.0,
    tspin_completeness: 0.0,
    average_h : -20.0,
    sum_attack: 0.0,
    sum_downstack: 35.0,
    attack: 0.0,
    downstack: 30.0,
    eff: 0.0,
};
const FACTORS_ATK: Factors = Factors {
    ideal_h: 0.0,
    well_threshold: 3.0,
};
const FACTORS_DS: Factors = Factors {
    ideal_h: 0.0,
    well_threshold: 20.0,
};
const CONSTS: Consts = Consts {
    ds_height_threshold: 14.0,
    ds_hole_threshold: 1.0,
    ds_mode_penalty: -1000.0,
    well_placement_f: 70.0,
    well_placement: [-1.0, -1.0, 0.8, 1.2, 1.0, 1.0, 1.2, 0.8, -1.0, -1.0],
};
const TSPIN_NEG: [u16; 3] = [
    0b110,
    0b111,
    0b010,
];

fn tspin_check (state: &State, x: usize, y: usize) -> Option<(u8, u8, usize, usize)> {
    // The x, y point given here is a hole (overhang'ed hole)
    let f: &Field = &state.field;
    if y < 1 || y > 18 { return None; }
        
    // println!("x: {}, y: {}", x, y);
    // Check overhang depth (must be 1)
    if f.m[y+1] & (1 << x) == 0 {
        return None;
    }
    // RIGHT
    let r: Option<(u8, usize, usize)> = if x <= 7 {'block: {
        // Check negative
        for j in 0..3 {
            //println!("row: {:010b} mask: {:010b}", f.m[y+j-1], TSPIN_NEG[j] << x);
            if TSPIN_NEG[j] << x & f.m[y+j-1] > 0 {
                break 'block None;
            }
        }
        // Check bottom far notch (impossible setup if only one is filled)
        if x < 7 && (f.m[y+1] & (1 << x+2) > 0) ^ ((f.m[y+1] & (1 << x+3) > 0)) {
            break 'block None;
        } else if f.m[y+1] & (1 << x+2) == 0 {
            break 'block None;
        }
        // println!("passed right @({}, {})", x, y);
        // check cleared rows
        Some((
            if f.m[y]   == ((1 << 10) - 1) ^ (0b111 << x) {1} else {0} + 
            if f.m[y+1] == ((1 << 10) - 1) ^ (0b010 << x) {1} else {0},
            x + 1, y
        ))
    }} else { None };

    // LEFT
    let l: Option<(u8, usize, usize)> = if x >= 2 {'block: {
        // Check negative
        for j in 0..3 {
            //println!("row: {:010b} mask: {:010b}", f.m[y+j-1], crate::field::reverse_bin(TSPIN_NEG[j], 3) << (x-2));
            if (crate::field::reverse_bin(TSPIN_NEG[j], 3) << (x-2)) & f.m[y+j-1] > 0 {
                break 'block None;
            }
        }
        // Check bottom far notch (impossible setup if only one is filled)
        if x > 2 && (f.m[y+1] & (1 << x-2) > 0) ^ (f.m[y+1] & (1 << x-3) > 0) {
            break 'block None;
        } else if f.m[y+1] & (1 << x-2) == 0 {
            break 'block None;
        }
        //println!("passed left @({}, {})", x, y);
        // check cleared rows
        Some((
            if f.m[y]   == ((1 << 10) - 1) ^ (0b111 << x-2) {1} else {0} + 
            if f.m[y+1] == ((1 << 10) - 1) ^ (0b010 << x-2) {1} else {0},
            x-1, y
        ))
    }} else { None };
    
    // Calculate dist from T piece.
    let depth = {
        let mut depth = if state.hold == Piece::T {1} else {7};
        for i in 0..state.pieces.len() {
            if state.pieces[i] == Piece::T {
                depth = i + 1;
                break;
            }
        }
        depth as u8
    };
    // No need to figure out which one is better, since it's unlikely that both will exist.
    if let Some(l) = l {Some((depth, l.0, l.1, l.2))} 
    else if let Some(r) = r {Some((depth, r.0, r.1, r.2))}
    else {None}
} 

/// Core heuristic function.
///
/// <TODO>: Heuristic list
pub fn evaluate (state: &State, mode: EvaluatorMode) -> f32 {
    let _bencher: Option<crate::Bencher> = if cfg!(feature = "bench") {
        unsafe {
            Some( crate::Bencher::new( &mut crate::BENCH_DATA.evaluator ) )
        }
    } else {None};

    let f: &Field = &state.field;
    let p: &Props = &state.props;
    let mut score: f32 = 0.0;
    const FW: usize = 10;
    const FH: usize = 20;
    let mut h: [u8; FW] = [0; FW];
    let mut well: Option<usize> = None;

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
    let (holes, hole_depth_sum_sq, tspin) = {
        let mut holes: f32 = 0.0;
        let mut depth_sum_sq: f32 = 0.0;
        let mut tspin: Option<(u8, u8, usize, usize)> = None; // (dist from T, clearable rows);

        for x in 0..FW {
            for y in (h[x] as usize + 1)..FH {
                if ( f.m[y] & ( 1 << x ) ) == 0 {
                    if let Some(_tspin) = tspin_check(&state, x, y) {
                        if let Some(ptspin) = tspin {
                            if _tspin.0 < ptspin.0 {
                                tspin = Some(_tspin)
                            }
                        } else {
                            tspin = Some(_tspin);
                        }
                    } else {
                        holes += 1.0;
                        let d: f32 = ((y - h[x] as usize) as f32).abs().min(3.0);
                        depth_sum_sq += d * d;
                    }
                }
            }
        }
        (holes, depth_sum_sq, tspin)
    };

    // Select weights
    // CURRENT SETTING: (for ds)
    // -> if holes
    // -> if average height past threshold
    let (weights, factors) = {
        match mode {
            EvaluatorMode::Norm => 
                if  FH as f32 - avg >= CONSTS.ds_height_threshold || holes >= CONSTS.ds_hole_threshold {
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
    // Score by tspin
    if let Some(tspin) = tspin {
        dev_log!(ln, "\x1b[1mtspin:\x1b[1m dist: {}, depth: {} @({}, {})", tspin.0, tspin.1, tspin.2, tspin.3);
        score += weights.tspin_flat_bonus;
        score += weights.tspin_dist * tspin.0 as f32;
        score += weights.tspin_completeness * tspin.1 as f32;
    }
    // Find well (max neg deviation from avg > than threshold)
    {
        for x in 0..10 {
            let d: f32 = avg - h[x] as f32;
            if d < 0.0 && d.abs() >= factors.well_threshold {
                if let Some(pwell) = well {
                    if avg - h[pwell as usize] as f32 > d {
                        well = Some(x);    
                    }
                } else {
                    well = Some(x);
                }
            }
        }
        if let Some(well) = well {
            avg = (avg * FW as f32 - h[well] as f32) / (FW - 1) as f32;
            dev_log!(ln, "identified well: \x1b[1m{}\x1b[0m", well);
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
                if w == x {
                    dev_log!("w ");
                    continue
                }
            }
            let d: f32 = avg - h[x] as f32;
            dev_log!("{} ", d);
            sum_sq += d * d;
        }
        // If tspin, compensate w/ [2, 1, 0]
        if tspin.is_some() {
            dev_log!("t-spin compensation: {}", 5.0 * weights.h_global_deviation);
            score -= 5.0 * weights.h_global_deviation;
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
            // Score well by height (not clear value)
            if let Some(w) = well { if x == w {
                let well_v = (0..20).fold(0, |y, _| if f.m[y] == ((1 << 10)-1) - (1 << w) {1} else {0});

                //let well_v = (if x != 0 {h[x-1]} else {20}).min(if x != 9 {h[x+1]} else {20}).abs_diff(h[x]);
                score += well_v as f32 * weights.well_v;
                score += CONSTS.well_placement_f * CONSTS.well_placement[x];
                dev_log!("w ");

                // Parity: penalize large parity diffs, bonus for flat well.
                let d = (if x != 0 {h[x-1]} else {h[x+1]}).abs_diff(if x != 9 {h[x+1]} else {h[x-1]});
                
                // Tspins: Subtract one from delta, due to inherent odd parity. 
                //         Promote an even-residue overhang for better contiuation.
                if let Some(tspin) = tspin { if tspin.2 == w { 
                    score -= 4.0 * weights.well_parity;
                    if d == 3 { score += weights.well_flat_parity }
                }}        
                score +=  (d * d) as f32 * weights.well_parity;
                if d % 2 == 1 { score += weights.well_odd_par };
                if d == 0 { score += weights.well_flat_parity }
                dev_log!("par: {} ", d);
                
                continue
            }}
            // If tspin, ignore, inherently bumpy
            if let Some(tspin) = tspin {
                if x.abs_diff(tspin.2) <= 1 {
                    dev_log!("t ");
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
        dev_log!(ln, "atk: {}, ds: {}; sum_atk: {}, sum_ds: {}", p.atk, p.ds, p.sum_atk, p.sum_ds);
        if p.sum_ds > 100 || p.sum_atk > 100 {
            println!("atk: {}, ds: {}; sum_atk: {}, sum_ds: {}", p.atk, p.ds, p.sum_atk, p.sum_ds);
        }
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
            0b0_0_0_0_0_0_0_0_0_0,
            0b0_0_0_0_1_1_0_0_0_0,
            0b0_0_0_0_0_1_1_1_1_1,
            0b0_1_1_1_1_1_1_1_1_1,
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
    
    #[test]
    fn tspin_check_test () {
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
            0b0_0_0_0_1_1_0_0_0_0,
            0b0_0_0_0_0_1_1_1_1_1,
            0b0_1_1_1_1_1_1_1_1_1,
        ];
        let mut state = State::new();
        
        state.field = field;
        //state.pieces.push_back(Piece::T);
        
        let out = {
            let f = &state.field;
            let mut out: Option<(u8, u8, usize, usize)> = None;
            for x in 0..10 {
                for y in 0..20 {
                    if ( f.m[y] & ( 1 << x ) ) == 0 {
                        if let Some(_tspin) = tspin_check(&state, x, y) {
                            if let Some(ptspin) = out {
                                if _tspin.0 < ptspin.0 {
                                    out = Some(_tspin)
                                }
                            } else {
                                out = Some(_tspin);
                            }
                        }
                    }
                }
            }
            out
        };
        print!("{:?}", out);
        /*
        assert!(out.is_some());
        let out = out.unwrap();
        assert_eq!(out.1, 2);
        assert_eq!(out.0, 1);
         */
    }
}
