use std::time::SystemTime;

use crate::parse;
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
    
    pub fn getname(&self) -> &str {
        &self.filename
    }
}

pub struct WatchScriptContext {
    pub filename: String,
    size: usize,
    fixtick: Option<u32>,

    watchtime: SystemTime,
    child: Box<RunContextWrap>,
}

impl WatchScriptContext {
    pub fn new(filename: &str, script: Script, size: usize, fixtick: Option<u32>) -> Result<WatchScriptContext, String> {
        let runner = ScriptRunner::new(script, &filename);
        let child = runner.build(size, fixtick)?;
        let stat = std::fs::metadata(filename)
            .map_err(|err| err.to_string())?;
        let watchtime = stat.modified()
            .map_err(|err| err.to_string())?;

        let ctx = WatchScriptContext {
            filename: filename.to_string(),
            size: size,
            fixtick: fixtick,
            watchtime: watchtime,
            child: Box::new(child),
        };
        Ok(ctx)
    }
}

impl RunContext for WatchScriptContext {

    fn tick(&mut self) -> Result<(), String> {
        let stat = std::fs::metadata(&self.filename)
            .map_err(|err| err.to_string())?;
        let newtime = stat.modified()
            .map_err(|err| err.to_string())?;
        if newtime != self.watchtime {
            println!("Reloading...");
            self.watchtime = newtime;
            match parse::parse_script(&self.filename) {
                Ok(newscript) => {
                    let newrunner = ScriptRunner::new(newscript, &self.filename);
                    let ctx = newrunner.build(self.size, self.fixtick)?;
                    self.child = Box::new(ctx);
                },
                Err(msg) => {
                    println!("{msg}");
                },
            }
        }
        
        self.child.tick()
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
