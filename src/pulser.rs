
use crate::context;

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
    duration: f32,
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
    pub fn new(ctx: &context::RunContext) -> Pulser {
        Pulser {
            birth: ctx.age(),
            nextpulse: 0.5,
            pulses: Vec::new(),
        }
    }

    pub fn tick(&mut self, ctx: &context::RunContext) {
        let age = ctx.age() - self.birth;
        if age >= self.nextpulse && self.pulses.is_empty() {
            self.pulses.push(Pulse {
                birth: ctx.age(),
                duration: 1.5,
                spaceshape:PulseShape::Triangle,
                timeshape:PulseShape::SawDecay,
                dead: false,
            });
        }

        self.pulses.retain(|pulse| !pulse.dead);
    }

    pub fn render(&mut self, ctx: &context::RunContext, buf: &mut [f32]) {
        let bufrange = buf.len() as f32;
        buf.fill(0.0);

        for pulse in &mut self.pulses {
            let mut timeval: f32 = 1.0;
            match pulse.timeshape {
                PulseShape::Flat => {},
                _ => {
                    let time = (ctx.age() - pulse.birth) as f32 / pulse.duration;
                    if time > 1.0 {
                        pulse.dead = true;
                        continue;
                    }
                    timeval = samplepulse(&pulse.timeshape, time);
                }
            }
            for ix in 0..buf.len() {
                let pos = (ix as f32) / bufrange;
                let spaceval = samplepulse(&pulse.spaceshape, pos);
                let val = spaceval * timeval;
                buf[ix] += val;
            }            
        }
    }
}
