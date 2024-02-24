use std::time::Instant;

pub struct CtxClock {
    pub fixtick: Option<u32>,
    
    birth: Instant,
    tickcount: usize,
    pub age: f64,
    pub ticklen: f32,
}

impl CtxClock {
    pub fn new(fixtick: Option<u32>) -> CtxClock {
        CtxClock {
            fixtick: fixtick,
            
            birth: Instant::now(),
            tickcount: 0,
            age: 0.0,
            ticklen: 0.0,
        }
    }

    pub fn tick(&mut self) -> f64 {
        let newage: f64;
        if let Some(fps) = &self.fixtick {
            newage = self.tickcount as f64 / *fps as f64;
        }
        else {
            let dur = self.birth.elapsed();
            newage = dur.as_secs_f64();
        }
        self.ticklen = (newage - self.age) as f32;
        self.age = newage;
        self.tickcount += 1;

        newage
    }
}
