use crate::script::Script;
use crate::runner::{Runner, RunContext, RunContextWrap, PixBuffer};
use crate::context::scriptcontext::{ScriptRunner, ScriptContext};

#[derive(Clone)]
pub struct WatchScriptRunner {
    pub filename: String,
    pub script: Script,
}

impl WatchScriptRunner {
    pub fn new(filename: &str, script: Script) -> Runner {
        let run = WatchScriptRunner {
            filename: filename.to_string(),
            script: script,
        };
        Runner::WatchScript(run)
    }
}

pub struct WatchScriptContext {
    pub filename: String,

    child: Box<RunContextWrap>,
}

impl WatchScriptContext {
    pub fn new(filename: &str, script: Script, size: usize, fixtick: Option<u32>) -> Result<WatchScriptContext, String> {
        let runner = ScriptRunner::new(script);
        let child = runner.build(size, fixtick)?;
        let ctx = WatchScriptContext {
            filename: filename.to_string(),
            child: Box::new(child),
        };
        Ok(ctx)
    }
}

impl RunContext for WatchScriptContext {

    fn tick(&mut self) {
        self.child.tick();
    }

    fn age(&self) -> f64 {
        self.child.age()
    }
    
    fn applybuf<F>(&self, func: F)
    where F: FnMut(PixBuffer) {
        self.child.applybuf(func);
    }

    fn done(&self) -> bool {
        self.child.done()
    }
    
}
