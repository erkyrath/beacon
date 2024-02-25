use std::fmt;
use std::cell::RefCell;
use rand::Rng;

use crate::context::scriptcontext::ScriptContext;
use crate::runner::RunContext;
use crate::lerp::Lerp;
use crate::pixel::Pix;
use crate::waves::WaveShape;
use crate::param::Param;
use crate::pulser::{Pulser, PulserState};
use crate::script::ScriptIndex;

#[derive(Clone)]
pub enum Op1Def {
    Constant(f32),
    Param(Param),
    Wave(WaveShape, Param, Param, Param, Param), // wave, min, max, pos, width
    WaveCycle(WaveShape, Param, Param, Param, Param), // wave, min, max, pos, period
    Invert(), // op1
    Pulser(Pulser),
    Decay(Param), // halflife; op1
    TimeDelta(), // op1
    Brightness(), // op3
    Gradient(Vec<f32>), // stops; op1
    Mul(), // op1, op1
    Sum(), // op1...
    Mean(), // op1...
    Min(), // op1...
    Max(), // op1...
    Clamp(Param, Param), // min, max; op1
    Shift(Param), // offset; op1
    ShiftDecay(Param, Param), // offset, halflife; op1
    Noise(usize, usize, Param, Param), // grain, octaves, offset, max
}

#[derive(Clone)]
pub enum Op3Def {
    Constant(Pix<f32>),
    Invert(), // op3
    Grey(), // op1
    RGB(), // op1, op1, op1
    HSV(), // op1, op1, op1
    HSVToRGB(), // op3
    RGBToHSV(), // op3
    Gradient(Vec<Pix<f32>>), // stops; op1
    PGradient(Vec<GradStop>), // stops; op1
    MulS(), // op3, op1
    Sum(), // op3...
    Mean(), // op3...
    Min(), // op3...
    Max(), // op3...
    Lerp(), // op3, op3, op1
    Mask(Param), // op3, op3, op1
    Shift(Param), // offset; op3
}

impl Op1Def {
    pub fn describe(&self, indent: Option<String>) -> String {
        match self {
            Op1Def::Constant(val) => {
                format!("Constant({})", val)
            },
            Op1Def::Param(val) => {
                format!("Param({:?})", val)
            },
            Op1Def::Wave(shape, min, max, pos, width) => {
                format!("Wave({:?}, min={:?}, max={:?}, pos={:?}, width={:?})", shape, min, max, pos, width)
            },
            Op1Def::WaveCycle(shape, min, max, pos, period) => {
                format!("Wave({:?}, min={:?}, max={:?}, pos={:?}, period={:?})", shape, min, max, pos, period)
            },
            Op1Def::Invert() => {
                format!("Invert()")
            },
            Op1Def::Pulser(pulser) => {
                let limitstr = if let Some(size) = pulser.countlimit {
                    format!(", countlimit={}", size)
                } else {
                    String::default()
                };
                let indentstr = if let Some(val) = indent {
                    val
                } else {
                    " ".to_string()
                };
                let desc = format!(
                    "Pulser(interval={:?}{},{}duration={:?},{}pos={:?},{}width={:?},{}spaceshape={:?}, timeshape={:?})",
                    pulser.interval, limitstr,
                    indentstr, pulser.duration,
                    indentstr, pulser.pos,
                    indentstr, pulser.width,
                    indentstr, pulser.spaceshape, pulser.timeshape);
                desc
            },
            Op1Def::Decay(halflife) => {
                format!("Decay({:?})", halflife)
            },
            Op1Def::TimeDelta() => {
                format!("TimeDelta()")
            },
            Op1Def::Brightness() => {
                format!("Brightness()")
            },
            Op1Def::Gradient(stops) => {
                let stopstrs = stops.iter().map(|stop| stop.to_string()).collect::<Vec<_>>();
                format!("Gradient({})", stopstrs.join(", "))
            },
            Op1Def::Mul() => {
                format!("Mul()")
            },
            Op1Def::Sum() => {
                format!("Sum()")
            },
            Op1Def::Mean() => {
                format!("Mean()")
            },
            Op1Def::Min() => {
                format!("Min()")
            },
            Op1Def::Max() => {
                format!("Max()")
            },
            Op1Def::Clamp(min, max) => {
                format!("Clamp({:?}, {:?})", min, max)
            },
            Op1Def::Shift(offset) => {
                format!("Shift({:?})", offset)
            },
            Op1Def::ShiftDecay(offset, halflife) => {
                format!("ShiftDecay(offset={:?}, halflife={:?})", offset, halflife)
            },
            Op1Def::Noise(grain, octaves, offset, max) => {
                format!("Noise(grain={}, octaves={}, offset={:?}, max={:?})", grain, octaves, offset, max)
            },
            //_ => "?Op1Def".to_string(),
        }
    }
}

impl Op3Def {
    pub fn describe(&self, _indent: Option<String>) -> String {
        match self {
            Op3Def::Constant(pix) => {
                format!("Constant(r={}, g={}, b={})", pix.r, pix.g, pix.b)
            },
            Op3Def::Invert() => {
                format!("Invert()")
            },
            Op3Def::Grey() => {
                format!("Grey()")
            },
            Op3Def::RGB() => {
                format!("RGB()")
            },
            Op3Def::HSV() => {
                format!("HSV()")
            },
            Op3Def::HSVToRGB() => {
                format!("HSVToRGB()")
            },
            Op3Def::RGBToHSV() => {
                format!("RGBToHSV()")
            },
            Op3Def::Gradient(stops) => {
                let stopstrs = stops.iter().map(|stop| stop.as_hex()).collect::<Vec<_>>();
                format!("Gradient({})", stopstrs.join(", "))
            },
            Op3Def::PGradient(stops) => {
                let stopstrs = stops.iter().map(|stop| format!("{}:{}", stop.pos, stop.color.as_hex())).collect::<Vec<_>>();
                format!("PGradient({})", stopstrs.join(", "))
            },
            Op3Def::MulS() => {
                format!("MulS()")
            },
            Op3Def::Sum() => {
                format!("Sum()")
            },
            Op3Def::Mean() => {
                format!("Mean()")
            },
            Op3Def::Min() => {
                format!("Min()")
            },
            Op3Def::Max() => {
                format!("Max()")
            },
            Op3Def::Lerp() => {
                format!("Lerp()")
            },
            Op3Def::Mask(threshold) => {
                format!("Mask({:?})", threshold)
            },
            Op3Def::Shift(offset) => {
                format!("Shift({:?})", offset)
            },
            //_ => "?Op3Def".to_string(),
        }
    }
}

impl fmt::Debug for Op1Def {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.describe(None))
    }
}

impl fmt::Debug for Op3Def {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.describe(None))
    }
}

pub enum Op1State {
    NoState,
    Pulser(PulserState),
    Decay(Vec<f32>),
    TimeDelta(Vec<f32>),
    Noise(NoiseState),
}

pub enum Op3State {
    NoState,
}

pub struct Op1Ctx {
    pub state: RefCell<Op1State>,
    pub buf: RefCell<Vec<f32>>,
}

pub struct Op3Ctx {
    pub state: RefCell<Op3State>,
    pub buf: RefCell<Vec<Pix<f32>>>,
}

pub struct NoiseState {
    seeds: Vec<Vec<f32>>,
    fudgemax: f32,
}

#[derive(Clone, Debug)]
pub struct GradStop {
    pub pos: f32,
    pub color: Pix<f32>,
}

impl NoiseState {
    pub fn new(grain: usize, octaves: usize, ctx: &mut ScriptContext) -> NoiseState {
        let mut res = NoiseState {
            seeds: Vec::default(),
            fudgemax: 1.0,
        };

        if octaves == 0 {
            return res;
        }
        
        let mut rng = ctx.rng.borrow_mut();
        let mut ograin = grain;
        let mut ofudge = 1.0;
        for _ in 0..octaves {
            let mut seed: Vec<f32> = Vec::default();
            for ix in 0..ograin {
                seed.push(ix as f32 / ograin as f32);
            }
            for ix in 0..ograin {
                let jx = rng.gen_range(0..ograin);
                seed.swap(ix, jx);
            }
            res.seeds.push(seed);
            ograin *= 2;
            ofudge /= 2.0;
        }

        res.fudgemax = 0.5 / (1.0 - ofudge);
        
        res
    }
}

impl Op1State {
    pub fn new_for(op: &Op1Def, ctx: &mut ScriptContext) -> Op1State {
        match op {
            Op1Def::Pulser(_pulser) => Op1State::Pulser(PulserState::new()),
            Op1Def::Decay(_halflife) => Op1State::Decay(vec![0.0; ctx.size()]),
            Op1Def::ShiftDecay(_offset, _halflife) => Op1State::Decay(vec![0.0; ctx.size()]),
            Op1Def::TimeDelta() => Op1State::TimeDelta(vec![0.0; ctx.size()]),
            Op1Def::Noise(grain, octaves, _offset, _max) => Op1State::Noise(NoiseState::new(*grain, *octaves, ctx)),
            _ => Op1State::NoState,
        }
    }
}

impl Op3State {
    pub fn new_for(op: &Op3Def, _ctx: &mut ScriptContext) -> Op3State {
        match op {
            _ => Op3State::NoState,
        }
    }
}

impl Op1Ctx {
    pub fn tickop(ctx: &mut ScriptContext, bufnum: usize) {
        let opref = &ctx.script.op1s[bufnum];
        let mut buf = ctx.op1s[bufnum].buf.borrow_mut();
        match &opref.op {
            Op1Def::Constant(val) => {
                for ix in 0..buf.len() {
                    buf[ix] = *val;
                }
            }

            Op1Def::Param(val) => {
                let age = ctx.age() as f32;
                let fval = val.eval(ctx, age);
                for ix in 0..buf.len() {
                    buf[ix] = fval;
                }
            }

            Op1Def::Wave(shape, min, max, pos, width) => {
                let age = ctx.age() as f32;
                let width = width.eval(ctx, age);
                let startpos = pos.eval(ctx, age) - width*0.5;
                let min = min.eval(ctx, age);
                let max = max.eval(ctx, age);
                let buflen32 = buf.len() as f32;
                for ix in 0..buf.len() {
                    let basepos = ix as f32 / buflen32;
                    buf[ix] = shape.sample((basepos-startpos) / width) * (max-min) + min;
                }
            }

            Op1Def::WaveCycle(shape, min, max, pos, period) => {
                let age = ctx.age() as f32;
                let period = period.eval(ctx, age);
                let startpos = pos.eval(ctx, age) - period*0.5;
                let min = min.eval(ctx, age);
                let max = max.eval(ctx, age);
                let buflen32 = buf.len() as f32;
                for ix in 0..buf.len() {
                    let basepos = ix as f32 / buflen32;
                    buf[ix] = shape.sample(((basepos-startpos) / period).rem_euclid(1.0)) * (max-min) + min;
                }
            }

            Op1Def::Invert() => {
                let obufnum = opref.get_type_ref(1, 0);
                let obuf = ctx.op1s[obufnum].buf.borrow();
                assert!(buf.len() == obuf.len());
                for ix in 0..buf.len() {
                    buf[ix] = 1.0 - obuf[ix];
                }
            }

            Op1Def::Pulser(pulser) => {
                let mut state = ctx.op1s[bufnum].state.borrow_mut();
                if let Op1State::Pulser(pstate) = &mut *state {
                    pstate.tick(ctx, &pulser);
                    pstate.render(ctx, &mut buf);
                }
                else {
                    panic!("Op1 state mismatch: PulserState");
                }
            }

            Op1Def::Decay(halflife) => {
                let age = ctx.age() as f32;
                let halflife = halflife.eval(ctx, age);
                let decaymul = (2.0_f32).powf(-ctx.ticklen()/halflife);
                let obufnum = opref.get_type_ref(1, 0);
                let obuf = ctx.op1s[obufnum].buf.borrow();
                let mut state = ctx.op1s[bufnum].state.borrow_mut();
                if let Op1State::Decay(historybuf) = &mut *state {
                    assert!(buf.len() == obuf.len());
                    assert!(buf.len() == historybuf.len());
                    for ix in 0..buf.len() {
                        let lastval = historybuf[ix];
                        historybuf[ix] = buf[ix];
                        buf[ix] = obuf[ix].max(lastval*decaymul);
                    }
                }
                else {
                    panic!("Op1 state mismatch: Decay");
                }
            }

            Op1Def::TimeDelta() => {
                let obufnum = opref.get_type_ref(1, 0);
                let obuf = ctx.op1s[obufnum].buf.borrow();
                let mut state = ctx.op1s[bufnum].state.borrow_mut();
                if let Op1State::TimeDelta(historybuf) = &mut *state {
                    assert!(buf.len() == obuf.len());
                    assert!(buf.len() == historybuf.len());
                    for ix in 0..buf.len() {
                        let lastval = historybuf[ix];
                        historybuf[ix] = obuf[ix];
                        buf[ix] = obuf[ix] - lastval;
                    }
                }
                else {
                    panic!("Op1 state mismatch: TimeDelta");
                }
            }

            Op1Def::Gradient(stops) => {
                let obufnum = opref.get_type_ref(1, 0);
                let obuf = ctx.op1s[obufnum].buf.borrow();
                assert!(buf.len() == obuf.len());
                let count = stops.len();
                if count == 0 {
                    for ix in 0..buf.len() {
                        buf[ix] = 0.0;
                    }
                }
                else if count == 1 {
                    for ix in 0..buf.len() {
                        buf[ix] = stops[0];
                    }
                }
                else {
                    for ix in 0..buf.len() {
                        if obuf[ix] < 0.0 {
                            buf[ix] = stops[0];
                        }
                        else {
                            let val = obuf[ix] * ((count-1) as f32);
                            let seg = val as usize;
                            let frac = val - (seg as f32);
                            if seg >= (count-1) {
                                buf[ix] = stops[count-1];
                            }
                            else {
                                buf[ix] = stops[seg].lerp(&stops[seg+1], &frac);
                            }
                        }
                    }
                }
            }

            Op1Def::Mul() => {
                let obufnum1 = opref.get_type_ref(1, 0);
                let obufnum2 = opref.get_type_ref(1, 1);
                let obuf1 = ctx.op1s[obufnum1].buf.borrow();
                let obuf2 = ctx.op1s[obufnum2].buf.borrow();
                assert!(buf.len() == obuf1.len());
                assert!(buf.len() == obuf2.len());
                for ix in 0..buf.len() {
                    buf[ix] = obuf1[ix]*obuf2[ix]
                }
            }
            
            Op1Def::Sum() => {
                if opref.bufs.len() == 0 {
                    for ix in 0..buf.len() {
                        buf[ix] = 0.0;
                    }
                }
                else {
                    let obufnum = opref.get_type_ref(1, 0);
                    let obuf1 = ctx.op1s[obufnum].buf.borrow();
                    for ix in 0..buf.len() {
                        buf[ix] = obuf1[ix];
                    }
                    for jx in 1..opref.bufs.len() {
                        let obufnum = opref.get_type_ref(1, jx);
                        let obuf = ctx.op1s[obufnum].buf.borrow();
                        for ix in 0..buf.len() {
                            buf[ix] += obuf[ix];
                        }
                    }
                }
            }
            
            Op1Def::Mean() => {
                if opref.bufs.len() == 0 {
                    for ix in 0..buf.len() {
                        buf[ix] = 0.0;
                    }
                }
                else if opref.bufs.len() == 1 {
                    let obufnum = opref.get_type_ref(1, 0);
                    let obuf1 = ctx.op1s[obufnum].buf.borrow();
                    for ix in 0..buf.len() {
                        buf[ix] = obuf1[ix];
                    }
                }
                else {
                    let obufnum = opref.get_type_ref(1, 0);
                    let obuf1 = ctx.op1s[obufnum].buf.borrow();
                    for ix in 0..buf.len() {
                        buf[ix] = obuf1[ix];
                    }
                    for jx in 1..opref.bufs.len() {
                        let obufnum = opref.get_type_ref(1, jx);
                        let obuf = ctx.op1s[obufnum].buf.borrow();
                        for ix in 0..buf.len() {
                            buf[ix] += obuf[ix];
                        }
                    }
                    for ix in 0..buf.len() {
                        buf[ix] /= opref.bufs.len() as f32;
                    }
                }
            }
            
            Op1Def::Min() => {
                if opref.bufs.len() == 0 {
                    for ix in 0..buf.len() {
                        buf[ix] = 0.0;
                    }
                }
                else if opref.bufs.len() == 1 {
                    let obufnum = opref.get_type_ref(1, 0);
                    let obuf1 = ctx.op1s[obufnum].buf.borrow();
                    for ix in 0..buf.len() {
                        buf[ix] = obuf1[ix];
                    }
                }
                else {
                    let obufnum = opref.get_type_ref(1, 0);
                    let obuf1 = ctx.op1s[obufnum].buf.borrow();
                    for ix in 0..buf.len() {
                        buf[ix] = obuf1[ix];
                    }
                    for jx in 1..opref.bufs.len() {
                        let obufnum = opref.get_type_ref(1, jx);
                        let obuf = ctx.op1s[obufnum].buf.borrow();
                        for ix in 0..buf.len() {
                            buf[ix] = buf[ix].min(obuf[ix]);
                        }
                    }
                }
            }
            
            Op1Def::Max() => {
                if opref.bufs.len() == 0 {
                    for ix in 0..buf.len() {
                        buf[ix] = 0.0;
                    }
                }
                else if opref.bufs.len() == 1 {
                    let obufnum = opref.get_type_ref(1, 0);
                    let obuf1 = ctx.op1s[obufnum].buf.borrow();
                    for ix in 0..buf.len() {
                        buf[ix] = obuf1[ix];
                    }
                }
                else {
                    let obufnum = opref.get_type_ref(1, 0);
                    let obuf1 = ctx.op1s[obufnum].buf.borrow();
                    for ix in 0..buf.len() {
                        buf[ix] = obuf1[ix];
                    }
                    for jx in 1..opref.bufs.len() {
                        let obufnum = opref.get_type_ref(1, jx);
                        let obuf = ctx.op1s[obufnum].buf.borrow();
                        for ix in 0..buf.len() {
                            buf[ix] = buf[ix].max(obuf[ix]);
                        }
                    }
                }
            }
            
            Op1Def::Clamp(min, max) => {
                let age = ctx.age() as f32;
                let min = min.eval(ctx, age);
                let max = max.eval(ctx, age);
                let obufnum = opref.get_type_ref(1, 0);
                let obuf = ctx.op1s[obufnum].buf.borrow();
                assert!(buf.len() == obuf.len());
                for ix in 0..buf.len() {
                    buf[ix] = obuf[ix].clamp(min, max);
                }
            }

            Op1Def::Shift(offset) => {
                let age = ctx.age() as f32;
                let offset = offset.eval(ctx, age);
                let buflen = buf.len() as i32;
                let buflen32 = buf.len() as f32;
                let obufnum = opref.get_type_ref(1, 0);
                let obuf = ctx.op1s[obufnum].buf.borrow();
                assert!(buf.len() == obuf.len());
                for ix in 0..buf.len() {
                    let pos = ix as f32 - offset * buflen32;
                    let seg = pos.floor() as i32;
                    let frac = pos - (seg as f32);
                    buf[ix] = obuf[seg.rem_euclid(buflen) as usize].lerp(&obuf[(seg+1).rem_euclid(buflen) as usize], &frac);
                }
            }

            Op1Def::ShiftDecay(offset, halflife) => {
                let age = ctx.age() as f32;
                let halflife = halflife.eval(ctx, age);
                let decaymul = (2.0_f32).powf(-ctx.ticklen()/halflife);
                let offset = offset.eval(ctx, age);
                let buflen = buf.len() as i32;
                let buflen32 = buf.len() as f32;
                let obufnum = opref.get_type_ref(1, 0);
                let obuf = ctx.op1s[obufnum].buf.borrow();
                let mut state = ctx.op1s[bufnum].state.borrow_mut();
                if let Op1State::Decay(historybuf) = &mut *state {
                    assert!(buf.len() == obuf.len());
                    assert!(buf.len() == historybuf.len());
                    for ix in 0..buf.len() {
                        let pos = ix as f32 - offset * buflen32;
                        let seg = pos.floor() as i32;
                        let frac = pos - (seg as f32);
                        // There's a speed bias here which I don't understand
                        let lastval = historybuf[seg.rem_euclid(buflen) as usize].lerp(&historybuf[(seg+1).rem_euclid(buflen) as usize], &frac);
                        historybuf[ix] = buf[ix];
                        buf[ix] = obuf[ix].max(lastval*decaymul);
                    }
                }
                else {
                    panic!("Op1 state mismatch: ShiftDecay");
                }
            }

            Op1Def::Noise(_grain, octaves, offset, max) => {
                let mut state = ctx.op1s[bufnum].state.borrow_mut();
                if let Op1State::Noise(state) = &mut *state {
                    let age = ctx.age() as f32;
                    let max = max.eval(ctx, age);
                    let offset = offset.eval(ctx, age);
                    let buflen32 = buf.len() as f32;
                    for ix in 0..buf.len() {
                        buf[ix] = 0.0;
                    }
                    for ix in 0..buf.len() {
                        let mut omax = max * state.fudgemax;
                        for oct in 0..*octaves {
                            let grain = state.seeds[oct].len() as i32;
                            let basepos = (ix as f32 / buflen32 - offset) * grain as f32;
                            let seg = basepos.floor() as i32;
                            let frac = basepos - (seg as f32);
                            let smoothfrac = (frac*frac)*(3.0-2.0*frac);
                            let val = state.seeds[oct][(seg.rem_euclid(grain)) as usize].lerp(&state.seeds[oct][((seg+1).rem_euclid(grain)) as usize], &smoothfrac);
                            buf[ix] += val * omax;
                            omax /= 2.0;
                        }
                    }
                }
                else {
                    panic!("Op1 state mismatch: Noise");
                }
            }
            
            _ => {
                panic!("unimplemented Op1");
            }
        }
    }
}

impl Op3Ctx {
    pub fn tickop(ctx: &mut ScriptContext, bufnum: usize) {
        let opref = &ctx.script.op3s[bufnum];
        //let mut _state = ctx.op3s[bufnum].state.borrow_mut();
        let mut buf = ctx.op3s[bufnum].buf.borrow_mut();
        match &opref.op {
            Op3Def::Constant(val) => {
                for ix in 0..buf.len() {
                    buf[ix] = val.clone();
                }
            }
            
            Op3Def::Invert() => {
                let obufnum = opref.get_type_ref(3, 0);
                let obuf = ctx.op3s[obufnum].buf.borrow();
                assert!(buf.len() == obuf.len());
                for ix in 0..buf.len() {
                    buf[ix] = Pix::new(1.0-obuf[ix].r, 1.0-obuf[ix].g, 1.0-obuf[ix].b);
                }
            }

            Op3Def::RGB() => {
                let obufnum1 = opref.get_type_ref(1, 0);
                let obufnum2 = opref.get_type_ref(1, 1);
                let obufnum3 = opref.get_type_ref(1, 2);
                let obuf1 = ctx.op1s[obufnum1].buf.borrow();
                let obuf2 = ctx.op1s[obufnum2].buf.borrow();
                let obuf3 = ctx.op1s[obufnum3].buf.borrow();
                assert!(buf.len() == obuf1.len());
                assert!(buf.len() == obuf2.len());
                assert!(buf.len() == obuf3.len());
                for ix in 0..buf.len() {
                    buf[ix] = Pix::new(obuf1[ix], obuf2[ix], obuf3[ix]);
                }
            }

            Op3Def::HSV() => {
                let obufnum1 = opref.get_type_ref(1, 0);
                let obufnum2 = opref.get_type_ref(1, 1);
                let obufnum3 = opref.get_type_ref(1, 2);
                let obuf1 = ctx.op1s[obufnum1].buf.borrow();
                let obuf2 = ctx.op1s[obufnum2].buf.borrow();
                let obuf3 = ctx.op1s[obufnum3].buf.borrow();
                assert!(buf.len() == obuf1.len());
                assert!(buf.len() == obuf2.len());
                assert!(buf.len() == obuf3.len());
                for ix in 0..buf.len() {
                    buf[ix] = Pix::from_hsv(obuf1[ix], obuf2[ix], obuf3[ix]);
                }
            }

            Op3Def::RGBToHSV() => {
                let obufnum = opref.get_type_ref(3, 0);
                let obuf = ctx.op3s[obufnum].buf.borrow();
                assert!(buf.len() == obuf.len());
                for ix in 0..buf.len() {
                    let (hue, sat, value) = obuf[ix].to_hsv();
                    buf[ix] = Pix::new(hue, sat, value);
                }
            }

            Op3Def::HSVToRGB() => {
                let obufnum = opref.get_type_ref(3, 0);
                let obuf = ctx.op3s[obufnum].buf.borrow();
                assert!(buf.len() == obuf.len());
                for ix in 0..buf.len() {
                    let (hue, sat, value) = (obuf[ix].r, obuf[ix].g, obuf[ix].b);
                    buf[ix] = Pix::from_hsv(hue, sat, value);
                }
            }

            Op3Def::Grey() => {
                let obufnum = opref.get_type_ref(1, 0);
                let obuf = ctx.op1s[obufnum].buf.borrow();
                assert!(buf.len() == obuf.len());
                for ix in 0..buf.len() {
                    buf[ix] = Pix::new(obuf[ix], obuf[ix], obuf[ix]);
                }
            }

            Op3Def::Gradient(stops) => {
                let obufnum = opref.get_type_ref(1, 0);
                let obuf = ctx.op1s[obufnum].buf.borrow();
                assert!(buf.len() == obuf.len());
                let count = stops.len();
                if count == 0 {
                    for ix in 0..buf.len() {
                        buf[ix] = Pix::new(0.0, 0.0, 0.0);
                    }
                }
                else if count == 1 {
                    for ix in 0..buf.len() {
                        buf[ix] = stops[0].clone();
                    }
                }
                else {
                    for ix in 0..buf.len() {
                        if obuf[ix] < 0.0 {
                            buf[ix] = stops[0].clone();
                        }
                        else {
                            let val = obuf[ix] * ((count-1) as f32);
                            let seg = val as usize;
                            let frac = val - (seg as f32);
                            if seg >= (count-1) {
                                buf[ix] = stops[count-1].clone();
                            }
                            else {
                                buf[ix] = stops[seg].lerp(&stops[seg+1], frac);
                            }
                        }
                    }
                }
            }

            Op3Def::PGradient(stops) => {
                let obufnum = opref.get_type_ref(1, 0);
                let obuf = ctx.op1s[obufnum].buf.borrow();
                assert!(buf.len() == obuf.len());
                let count = stops.len();
                if count == 0 {
                    for ix in 0..buf.len() {
                        buf[ix] = Pix::new(0.0, 0.0, 0.0);
                    }
                }
                else if count == 1 {
                    for ix in 0..buf.len() {
                        buf[ix] = stops[0].color.clone();
                    }
                }
                else {
                    for ix in 0..buf.len() {
                        let seg = stops.partition_point(|stop| stop.pos < obuf[ix]);
                        if seg == 0 {
                            buf[ix] = stops[0].color.clone();
                        }
                        else if seg >= count {
                            buf[ix] = stops[count-1].color.clone();
                        }
                        else {
                            let frac = (obuf[ix] - stops[seg-1].pos) / (stops[seg].pos - stops[seg-1].pos);
                            buf[ix] = stops[seg-1].color.lerp(&stops[seg].color, frac);
                        }
                    }
                }
            },
            
            Op3Def::MulS() => {
                let obufnum1 = opref.get_type_ref(3, 0);
                let obufnum2 = opref.get_type_ref(1, 1);
                let obuf1 = ctx.op3s[obufnum1].buf.borrow();
                let obuf2 = ctx.op1s[obufnum2].buf.borrow();
                assert!(buf.len() == obuf1.len());
                assert!(buf.len() == obuf2.len());
                for ix in 0..buf.len() {
                    buf[ix] = Pix::new(obuf1[ix].r*obuf2[ix], obuf1[ix].g*obuf2[ix], obuf1[ix].b*obuf2[ix]);
                }
            }

            Op3Def::Sum() => {
                if opref.bufs.len() == 0 {
                    for ix in 0..buf.len() {
                        buf[ix] = Pix::new(0.0, 0.0, 0.0);
                    }
                }
                else {
                    let obufnum = opref.get_type_ref(3, 0);
                    let obuf1 = ctx.op3s[obufnum].buf.borrow();
                    for ix in 0..buf.len() {
                        buf[ix] = obuf1[ix].clone();
                    }
                    for jx in 1..opref.bufs.len() {
                        let obufnum = opref.get_type_ref(3, jx);
                        let obuf = ctx.op3s[obufnum].buf.borrow();
                        for ix in 0..buf.len() {
                            buf[ix].r += obuf[ix].r;
                            buf[ix].g += obuf[ix].g;
                            buf[ix].b += obuf[ix].b;
                        }
                    }
                }
            }
            
            Op3Def::Mean() => {
                if opref.bufs.len() == 0 {
                    for ix in 0..buf.len() {
                        buf[ix] = Pix::new(0.0, 0.0, 0.0);
                    }
                }
                else if opref.bufs.len() == 1 {
                    let obufnum = opref.get_type_ref(3, 0);
                    let obuf1 = ctx.op3s[obufnum].buf.borrow();
                    for ix in 0..buf.len() {
                        buf[ix] = obuf1[ix].clone();
                    }
                }
                else {
                    let obufnum = opref.get_type_ref(3, 0);
                    let obuf1 = ctx.op3s[obufnum].buf.borrow();
                    for ix in 0..buf.len() {
                        buf[ix] = obuf1[ix].clone();
                    }
                    for jx in 1..opref.bufs.len() {
                        let obufnum = opref.get_type_ref(3, jx);
                        let obuf = ctx.op3s[obufnum].buf.borrow();
                        for ix in 0..buf.len() {
                            buf[ix].r += obuf[ix].r;
                            buf[ix].g += obuf[ix].g;
                            buf[ix].b += obuf[ix].b;
                        }
                    }
                    for ix in 0..buf.len() {
                        buf[ix].r /= opref.bufs.len() as f32;
                        buf[ix].g /= opref.bufs.len() as f32;
                        buf[ix].b /= opref.bufs.len() as f32;
                    }
                }
            }
            
            Op3Def::Min() => {
                if opref.bufs.len() == 0 {
                    for ix in 0..buf.len() {
                        buf[ix] = Pix::new(0.0, 0.0, 0.0);
                    }
                }
                else if opref.bufs.len() == 1 {
                    let obufnum = opref.get_type_ref(3, 0);
                    let obuf1 = ctx.op3s[obufnum].buf.borrow();
                    for ix in 0..buf.len() {
                        buf[ix] = obuf1[ix].clone();
                    }
                }
                else {
                    let obufnum = opref.get_type_ref(3, 0);
                    let obuf1 = ctx.op3s[obufnum].buf.borrow();
                    for ix in 0..buf.len() {
                        buf[ix] = obuf1[ix].clone();
                    }
                    for jx in 1..opref.bufs.len() {
                        let obufnum = opref.get_type_ref(3, jx);
                        let obuf = ctx.op3s[obufnum].buf.borrow();
                        for ix in 0..buf.len() {
                            buf[ix].r = buf[ix].r.min(obuf[ix].r);
                            buf[ix].g = buf[ix].g.min(obuf[ix].g);
                            buf[ix].b = buf[ix].b.min(obuf[ix].b);
                        }
                    }
                }
            }
            
            Op3Def::Max() => {
                if opref.bufs.len() == 0 {
                    for ix in 0..buf.len() {
                        buf[ix] = Pix::new(0.0, 0.0, 0.0);
                    }
                }
                else if opref.bufs.len() == 1 {
                    let obufnum = opref.get_type_ref(3, 0);
                    let obuf1 = ctx.op3s[obufnum].buf.borrow();
                    for ix in 0..buf.len() {
                        buf[ix] = obuf1[ix].clone();
                    }
                }
                else {
                    let obufnum = opref.get_type_ref(3, 0);
                    let obuf1 = ctx.op3s[obufnum].buf.borrow();
                    for ix in 0..buf.len() {
                        buf[ix] = obuf1[ix].clone();
                    }
                    for jx in 1..opref.bufs.len() {
                        let obufnum = opref.get_type_ref(3, jx);
                        let obuf = ctx.op3s[obufnum].buf.borrow();
                        for ix in 0..buf.len() {
                            buf[ix].r = buf[ix].r.max(obuf[ix].r);
                            buf[ix].g = buf[ix].g.max(obuf[ix].g);
                            buf[ix].b = buf[ix].b.max(obuf[ix].b);
                        }
                    }
                }
            }

            Op3Def::Lerp() => {
                let obufnum1 = opref.get_type_ref(3, 0);
                let obufnum2 = opref.get_type_ref(3, 1);
                let obufnum3 = opref.get_type_ref(1, 2);
                let obuf1 = ctx.op3s[obufnum1].buf.borrow();
                let obuf2 = ctx.op3s[obufnum2].buf.borrow();
                let obuf3 = ctx.op1s[obufnum3].buf.borrow();
                assert!(buf.len() == obuf1.len());
                assert!(buf.len() == obuf2.len());
                assert!(buf.len() == obuf3.len());
                for ix in 0..buf.len() {
                    buf[ix] = obuf1[ix].lerp(&obuf2[ix], obuf3[ix]);
                }
            }
            
            Op3Def::Mask(threshold) => {
                let age = ctx.age() as f32;
                let thresval = threshold.eval(ctx, age);
                let obufnum1 = opref.get_type_ref(3, 0);
                let obufnum2 = opref.get_type_ref(3, 1);
                let obufnum3 = opref.get_type_ref(1, 2);
                let obuf1 = ctx.op3s[obufnum1].buf.borrow();
                let obuf2 = ctx.op3s[obufnum2].buf.borrow();
                let obuf3 = ctx.op1s[obufnum3].buf.borrow();
                assert!(buf.len() == obuf1.len());
                assert!(buf.len() == obuf2.len());
                assert!(buf.len() == obuf3.len());
                for ix in 0..buf.len() {
                    if obuf3[ix] < thresval {
                        buf[ix] = obuf1[ix].clone();
                    }
                    else {
                        buf[ix] = obuf2[ix].clone();
                    }
                }
            }

            Op3Def::Shift(offset) => {
                let age = ctx.age() as f32;
                let offset = offset.eval(ctx, age);
                let buflen = buf.len() as i32;
                let buflen32 = buf.len() as f32;
                let obufnum = opref.get_type_ref(3, 0);
                let obuf = ctx.op3s[obufnum].buf.borrow();
                assert!(buf.len() == obuf.len());
                for ix in 0..buf.len() {
                    let pos = ix as f32 - offset * buflen32;
                    let seg = pos.floor() as i32;
                    let frac = pos - (seg as f32);
                    buf[ix].r = obuf[seg.rem_euclid(buflen) as usize].r.lerp(&obuf[(seg+1).rem_euclid(buflen) as usize].r, &frac);
                    buf[ix].g = obuf[seg.rem_euclid(buflen) as usize].g.lerp(&obuf[(seg+1).rem_euclid(buflen) as usize].g, &frac);
                    buf[ix].b = obuf[seg.rem_euclid(buflen) as usize].b.lerp(&obuf[(seg+1).rem_euclid(buflen) as usize].b, &frac);
                }
            }

            //_ => { panic!("unimplemented Op3"); }
        }
    }
}
