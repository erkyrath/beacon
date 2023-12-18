use std::time::Instant;

pub struct RunContext {
    birth: Instant,
    age: f64,
}

impl RunContext {
    pub fn new() -> RunContext {
        RunContext {
            birth: Instant::now(),
            age: 0.0,
        }
    }

    pub fn age(&self) -> f64 {
        self.age
    }
    
    pub fn tick(&mut self) {
        let dur = self.birth.elapsed();
        self.age = dur.as_secs_f64();
    }

}
