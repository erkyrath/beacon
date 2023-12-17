
#[derive(Debug, Clone)]
pub enum PulseShape {
    Flat,
    Square,
    Triangle,
    SawTooth,
}

pub struct Pulse {
    pub spaceshape: PulseShape,
    pub timeshape: PulseShape,
}

pub struct Pulser {
    pulses: Vec<Pulse>,
}

impl Pulser {
    pub fn new() -> Pulser {
        Pulser {
            pulses: Vec::new(),
        }
    }

    pub fn render(&self, buf: &mut [f32]) {
        let bufrange = buf.len() as f32;
        buf.fill(0.0);
        if !self.pulses.is_empty() {
            for ix in 0..buf.len() {
                let pos = (ix as f32) / bufrange;
                
            }
        }
    }
}
