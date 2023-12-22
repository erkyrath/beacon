use rand::Rng;

use crate::context::RunContext;
use crate::param::Param;

#[derive(Debug, Clone)]
pub enum PulseShape {
    Flat,
    Square,
    Triangle,
    SawTooth,
    SqrTooth,
    SawDecay,
    SqrDecay,
    Sine,
}

fn samplepulse(shape: &PulseShape, pos: f32) -> f32 {
    match shape {
        PulseShape::Flat => 1.0,
        PulseShape::Square => {
            if pos >= 0.0 && pos < 1.0 {
                1.0
            }
            else {
                0.0
            }
        },
        PulseShape::SawTooth => {
            if pos >= 0.0 && pos < 1.0 {
                pos
            }
            else {
                0.0
            }
        },
        PulseShape::SqrTooth => {
            if pos >= 0.0 && pos < 1.0 {
                pos*pos
            }
            else {
                0.0
            }
        },
        PulseShape::SawDecay => {
            if pos >= 0.0 && pos < 1.0 {
                1.0 - pos
            }
            else {
                0.0
            }
        },
        PulseShape::SqrDecay => {
            if pos >= 0.0 && pos < 1.0 {
                (1.0-pos)*(1.0-pos)
            }
            else {
                0.0
            }
        },
        PulseShape::Triangle => {
            if pos >= 0.0 && pos < 0.5 {
                pos * 2.0
            }
            else if pos >= 0.5 && pos < 1.0 {
                (1.0 - pos) * 2.0
            }
            else {
                0.0
            }
        },
        PulseShape::Sine => {
            if pos >= 0.0 && pos < 1.0 {
                0.5 - 0.5 * (2.0*std::f32::consts::PI*pos).cos()
            }
            else {
                0.0
            }
        },
    }                
}

pub struct Pulser {
    //### pulse def info...
}

impl Pulser {
    pub fn new() -> Pulser {
        Pulser {}
    }
}

pub struct Pulse {
    birth: f64,
    duration: Param,
    pos: Param,
    width: f32,
    velocity: f32,
    pub spaceshape: PulseShape,
    pub timeshape: PulseShape,
    dead: bool,
}

pub struct PulserState {
    birth: f64,
    nextpulse: f64,
    interval: Param,
    pulses: Vec<Pulse>,
}

impl PulserState {
    pub fn new() -> PulserState {
        PulserState {
            birth: 0.0, // not handling on-the-fly pulsers yet
            nextpulse: 0.0,
            interval: Param::Constant(0.4),
            pulses: Vec::new(),
        }
    }

    pub fn tick(&mut self, ctx: &RunContext) {
        let age = ctx.age() - self.birth;
        if age >= self.nextpulse {
            //let dur = eval(&Param::RandFlat(1.0, 5.0), ctx, age as f32);
            let pos = Param::RandNorm(0.5, 0.3).eval(ctx, age as f32);
            self.pulses.push(Pulse {
                birth: ctx.age(),
                duration: Param::Constant(2.0),
                pos: Param::Constant(pos),
                width: 0.3,
                velocity: 0.1,
                spaceshape:PulseShape::Triangle,
                timeshape:PulseShape::SqrDecay,
                dead: false,
            });

            self.nextpulse = ctx.age() + self.interval.eval(ctx, age as f32) as f64;
        }

        self.pulses.retain(|pulse| !pulse.dead);
    }

    pub fn render(&mut self, ctx: &RunContext, buf: &mut [f32]) {
        let bufrange = buf.len() as f32;
        buf.fill(0.0);

        for pulse in &mut self.pulses {
            let age = (ctx.age() - pulse.birth) as f32;
            let timeval: f32;
            match pulse.timeshape {
                PulseShape::Flat => {
                    timeval = 1.0;
                },
                _ => {
                    let duration = pulse.duration.eval(ctx, age);
                    let time = age / duration;
                    if time > 1.0 {
                        pulse.dead = true;
                        continue;
                    }
                    timeval = samplepulse(&pulse.timeshape, time);
                }
            }
            
            let startpos: f32;
            match pulse.spaceshape {
                PulseShape::Flat => {
                    startpos = 0.0;
                },
                _ => {
                    startpos = pulse.pos.eval(ctx, age) - pulse.width*0.5 + age * pulse.velocity;
                    if pulse.velocity >= 0.0 && startpos > 1.0 {
                        pulse.dead = true;
                        continue;
                    }
                    if pulse.velocity <= 0.0 && startpos+pulse.width < 0.0 {
                        pulse.dead = true;
                        continue;
                    }
                }
            }
            
            for ix in 0..buf.len() {
                let spaceval: f32;
                match pulse.spaceshape {
                    PulseShape::Flat => {
                        spaceval = 1.0;
                    },
                    _ => {
                        let pos = (ix as f32) / bufrange;
                        let rpos = (pos - startpos) / pulse.width;
                        spaceval = samplepulse(&pulse.spaceshape, rpos);
                    }
                }
                let val = spaceval * timeval;
                buf[ix] += val;
            }            
        }
    }
}
