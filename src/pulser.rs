use rand::Rng;

use crate::context::RunContext;
use crate::param::Param;
use crate::waves::WaveShape;

pub struct Pulser {
    pub interval: Param,
    pub countlimit: Option<usize>,
    pub duration: Param,
    pub pos: Param,
    pub width: Param,
    pub spaceshape: WaveShape,
    pub timeshape: WaveShape,
}

impl Pulser {
    pub fn new() -> Pulser {
        Pulser {
            interval: Param::Constant(1.0),
            countlimit: None,
            duration: Param::Constant(1.0),
            pos: Param::Constant(0.5),
            width: Param::Constant(0.5),
            spaceshape: WaveShape::Triangle,
            timeshape: WaveShape::SqrDecay,
        }
    }
}

pub struct Pulse {
    birth: f64,
    duration: Param,
    pos: Param,
    width: Param,
    spaceshape: WaveShape,
    timeshape: WaveShape,
    dead: bool,
}

pub struct PulserState {
    birth: f64,
    nextpulse: f64,
    totalcount: usize,
    pulses: Vec<Pulse>,
}

impl PulserState {
    pub fn new() -> PulserState {
        PulserState {
            birth: 0.0, // not handling on-the-fly pulsers yet
            nextpulse: 0.0,
            totalcount: 0,
            pulses: Vec::new(),
        }
    }

    pub fn tick(&mut self, ctx: &RunContext, pulser: &Pulser) {
        let age = ctx.age() - self.birth;
        if age >= self.nextpulse {
            let posparam = pulser.pos.resolve(ctx, age as f32);
            let widthparam = pulser.width.resolve(ctx, age as f32);
            let durparam = pulser.duration.resolve(ctx, age as f32);
            self.pulses.push(Pulse {
                birth: ctx.age(),
                duration: durparam,
                pos: posparam,
                width: widthparam,
                spaceshape: pulser.spaceshape,
                timeshape: pulser.timeshape,
                dead: false,
            });

            self.totalcount += 1;
            self.nextpulse = ctx.age() + pulser.interval.eval(ctx, age as f32) as f64;
            if let Some(countlimit) = pulser.countlimit {
                if self.totalcount >= countlimit {
                    self.nextpulse = f64::INFINITY;
                }
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
                WaveShape::Flat => {
                    timeval = 1.0;
                },
                _ => {
                    let duration = pulse.duration.eval(ctx, age);
                    let time = age / duration;
                    if time > 1.0 {
                        pulse.dead = true;
                        continue;
                    }
                    timeval = pulse.timeshape.sample(time);
                }
            }
            
            let width: f32 = pulse.width.eval(ctx, age);
            let startpos: f32;
            match pulse.spaceshape {
                WaveShape::Flat => {
                    startpos = 0.0;
                },
                _ => {
                    startpos = pulse.pos.eval(ctx, age) - width*0.5;
                    if let Some(minpos) = pulse.pos.min(ctx, age) {
                        if minpos - width*0.5 > 1.0 {
                            pulse.dead = true;
                        }
                    }
                    if let Some(maxpos) = pulse.pos.max(ctx, age) {
                        if maxpos + width*0.5 < 0.0 {
                            pulse.dead = true;
                        }
                    }
                }
            }

            if pulse.dead {
                continue;
            }
            
            for ix in 0..buf.len() {
                let spaceval: f32;
                match pulse.spaceshape {
                    WaveShape::Flat => {
                        spaceval = 1.0;
                    },
                    _ => {
                        let pos = (ix as f32) / bufrange;
                        let rpos = (pos - startpos) / width;
                        spaceval = pulse.spaceshape.sample(rpos);
                    }
                }
                let val = spaceval * timeval;
                buf[ix] += val;
            }            
        }
    }
}
