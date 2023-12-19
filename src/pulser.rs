use rand::Rng;

use crate::context::RunContext;
use crate::param::Param;
use crate::param::eval;

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

pub struct Pulse {
    birth: f64,
    duration: Param,
    startpos: f32,
    width: f32,
    velocity: f32,
    pub spaceshape: PulseShape,
    pub timeshape: PulseShape,
    dead: bool,
}

pub struct Pulser {
    birth: f64,
    nextpulse: f64,
    pulses: Vec<Pulse>,
}

impl Pulser {
    pub fn new(ctx: &RunContext) -> Pulser {
        Pulser {
            birth: ctx.age(),
            nextpulse: 0.0,
            pulses: Vec::new(),
        }
    }

    pub fn tick(&mut self, ctx: &RunContext) {
        let age = ctx.age() - self.birth;
        if age >= self.nextpulse {
            self.pulses.push(Pulse {
                birth: ctx.age(),
                duration: Param::Constant(3.0),
                startpos: -0.2,
                width: 0.2,
                velocity: 0.5,
                spaceshape:PulseShape::SawTooth,
                timeshape:PulseShape::SqrDecay,
                dead: false,
            });

            {
                let mut rng = ctx.rng.borrow_mut();
                self.nextpulse = ctx.age() + rng.gen_range(0.1..0.4);
            }
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
                    let duration = eval(&pulse.duration, ctx, age);
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
                    startpos = pulse.startpos + age * pulse.velocity;
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
