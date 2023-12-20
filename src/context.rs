use std::time::Instant;
use std::rc::Rc;
use std::cell::RefCell;
use rand::rngs::SmallRng;
use rand::SeedableRng;

pub struct RunContext {
    size: usize,
    birth: Instant,
    age: f64,
    pub rng: Rc<RefCell<SmallRng>>,
}

impl RunContext {
    pub fn new(size: usize) -> RunContext {
        RunContext {
            size: size,
            birth: Instant::now(),
            age: 0.0,
            rng: Rc::new(RefCell::new(SmallRng::from_entropy())),
        }
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn age(&self) -> f64 {
        self.age
    }
    
    pub fn tick(&mut self) {
        let dur = self.birth.elapsed();
        self.age = dur.as_secs_f64();
    }

}
