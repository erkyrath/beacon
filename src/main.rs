#![allow(dead_code)]
#![allow(unused_imports)]

use gumdrop::Options;
extern crate sdl2; 
#[macro_use]
extern crate lazy_static;

use std::time::Duration;

use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;

mod pixel;
mod op;
mod script;
mod parse;
mod param;
mod context;
mod waves;
mod pulser;

use script::{Script, ScriptIndex};

#[derive(Options, Debug)]
pub struct AppOptions {
    #[options(free)]
    args: Vec<String>,
    
    #[options(help = "print help message")]
    help: bool,

    #[options(long="dump", help = "dump script to stdout")]
    dump: bool,

    #[options(long="spin", help = "run script headless and measure speed")]
    spin: bool,

}

fn main() {
    let opts = AppOptions::parse_args_default_or_exit();

    if opts.args.len() != 1 {
        println!("usage: beacon [--dump] script");
        return;
    }

    let filename = &opts.args[0];
    let script: Script;
    
    match parse::parse_script(&filename) {
        Ok(val) => {
            script = val;
        },
        Err(msg) => {
            println!("{msg}");
            return;
        },
    }

    if opts.dump {
        script.dump();
    }
    else if opts.spin {
        let dur: f64 = 0.1;
        let res = run_spin(script, dur);
        match res {
            Err(msg) => {
                println!("{msg}");
            },
            Ok(count) => {
                println!("{} frames in {} seconds", count, dur);
            },
        }
    }
    else {
        let res = run_sdl(script);
        if let Err(msg) = res {
            println!("{msg}");
        }
    }
}

fn run_spin(script: Script, seconds: f64) -> Result<usize, String> {
    let pixsize: usize = 160;
    let mut ctx = context::RunContext::new(script, pixsize);
    let mut count = 0;
    
    loop {
        ctx.tick();
        count += 1;
        if ctx.age() > seconds {
            break;
        }
    }
    
    Ok(count)
}

fn run_sdl(script: Script) -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
 
    let window = video_subsystem.window("beacon demo", 800, 100)
        .position_centered()
        .build()
        .map_err(|err| err.to_string())?;

    let pixsize: usize = 160;
    
    let mut canvas = window.into_canvas().build()
        .map_err(|err| err.to_string())?;
    let tc = canvas.texture_creator();
    let mut texture = tc.create_texture_streaming(PixelFormatEnum::RGB24, pixsize as u32, 1)
        .map_err(|err| err.to_string())?;
 
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();
    
    let mut event_pump = sdl_context.event_pump()?;

    let mut ctx = context::RunContext::new(script, pixsize);
        
    'running: loop {
        ctx.tick();

        texture.with_lock(None, |buffer: &mut [u8], _pitch: usize| {
            match &ctx.script.order[0] {
                ScriptIndex::Op1(val) => {
                    ctx.applybuf1(*val, |buf| {
                        for xpos in 0..pixsize {
                            let offset = (xpos as usize) * 3;
                            buffer[offset] = (buf[xpos] * 255.0) as u8;
                            buffer[offset+1] = buffer[offset];
                            buffer[offset+2] = buffer[offset];
                        }
                    });
                },
                ScriptIndex::Op3(val) => {
                    ctx.applybuf3(*val, |buf| {
                        for xpos in 0..pixsize {
                            let offset = (xpos as usize) * 3;
                            buffer[offset] = (buf[xpos].r * 255.0) as u8;
                            buffer[offset+1] = (buf[xpos].g * 255.0) as u8;
                            buffer[offset+2] = (buf[xpos].b * 255.0) as u8;
                        }
                    });
                },
            }
        })?;
        canvas.copy(&texture, None, None)?;
        
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running Ok(())
                },
                _ => {}
            }
        }

        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
        
    }
}
