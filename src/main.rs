#![allow(dead_code)]
#![allow(unused_imports)]

use gumdrop::Options;
extern crate sdl2; 
#[macro_use]
extern crate lazy_static;

use std::time::Duration;
use std::time::SystemTime;

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
use context::{RunContext, PixBuffer};

#[derive(Options, Debug)]
pub struct AppOptions {
    #[options(free)]
    args: Vec<String>,
    
    #[options(help = "print help message")]
    help: bool,

    #[options(long="size", help = "pixel count (default 160)")]
    size: Option<usize>,

    #[options(long="dump", help = "dump script to stdout")]
    dump: bool,

    #[options(long="spin", help = "run script headless and measure speed")]
    spin: bool,

    #[options(long="watch", help = "watch script and reload if it changes")]
    watchfile: bool,

    #[options(long="power", help = "estimate power usage")]
    showpower: bool,

    #[options(long="width", help = "display window width")]
    winwidth: Option<u32>,

    #[options(long="height", help = "display window height")]
    winheight: Option<u32>,

}

fn main() {
    let opts = AppOptions::parse_args_default_or_exit();

    if opts.args.len() != 1 {
        println!("usage: beacon [--dump] script");
        return;
    }

    let pixsize = match opts.size {
        Some(val) => val,
        None => 160,
    };

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
        let res = script.consistency_check();
        match res {
            Err(msg) => {
                println!("{msg}");
            },
            Ok(()) => {},
        }
    }
    else if opts.spin {
        let dur: f64 = 0.1;
        let res = run_spin(script, pixsize, dur);
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
        let winwidth = opts.winwidth.unwrap_or(800);
        let winheight = opts.winheight.unwrap_or(100);
        let res = run_sdl(script, pixsize, filename, opts.watchfile, opts.showpower, winwidth, winheight);
        if let Err(msg) = res {
            println!("{msg}");
        }
    }
}

fn run_spin(script: Script, pixsize: usize, seconds: f64) -> Result<usize, String> {
    let mut ctx = RunContext::new(script, pixsize);
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

fn run_sdl(script: Script, pixsize: usize, filename: &str, watchfile: bool, showpower: bool, winwidth: u32, winheight: u32) -> Result<(), String> {
    script.consistency_check()?;
    
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let mut powertime: f64 = 0.0;
 
    let mut watchtime: SystemTime = SystemTime::now();
    if watchfile {
        let stat = std::fs::metadata(filename)
            .map_err(|err| err.to_string())?;
        watchtime = stat.modified()
            .map_err(|err| err.to_string())?;
    }

    let window = video_subsystem.window(format!("beacon: {}", filename).as_str(), winwidth, winheight)
        .position_centered()
        .build()
        .map_err(|err| err.to_string())?;

    let mut canvas = window.into_canvas().build()
        .map_err(|err| err.to_string())?;
    let tc = canvas.texture_creator();
    let mut texture = tc.create_texture_streaming(PixelFormatEnum::RGB24, pixsize as u32, 1)
        .map_err(|err| err.to_string())?;
 
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();
    
    let mut event_pump = sdl_context.event_pump()?;

    let mut ctx = RunContext::new(script, pixsize);
    let mut pause = false;
        
    'running: loop {
        if watchfile {
            let stat = std::fs::metadata(filename)
                .map_err(|err| err.to_string())?;
            let newtime = stat.modified()
                .map_err(|err| err.to_string())?;
            if newtime != watchtime {
                println!("Reloading...");
                watchtime = newtime;
                match parse::parse_script(&filename) {
                    Ok(newscript) => {
                        ctx = RunContext::new(newscript, pixsize);
                    },
                    Err(msg) => {
                        println!("{msg}");
                    },
                }
            }
        }

        if !pause {
            ctx.tick();
        }

        if showpower {
            if ctx.age() >= powertime+1.0 {
                let mut total = 0.0;
                ctx.applybuf(|pixbuf| {
                    match pixbuf {
                        PixBuffer::Buf1(buf) => {
                            for xpos in 0..pixsize {
                                total += buf[xpos];
                            }
                        },
                        PixBuffer::Buf3(buf) => {
                            for xpos in 0..pixsize {
                                total += (buf[xpos].r + buf[xpos].g + buf[xpos].b) / 3.0;
                            }
                        },
                    }
                });
                println!("Power use: {:.1} white pixels ({:.01}%)", total, 100.0 * total / (pixsize as f32));
                powertime = ctx.age();
            }
        }
        
        texture.with_lock(None, |buffer: &mut [u8], _pitch: usize| {
            ctx.applybuf(|pixbuf| {
                match pixbuf {
                    PixBuffer::Buf1(buf) => {
                        for xpos in 0..pixsize {
                            let offset = (xpos as usize) * 3;
                            buffer[offset] = (buf[xpos] * 255.0) as u8;
                            buffer[offset+1] = buffer[offset];
                            buffer[offset+2] = buffer[offset];
                        }
                    },
                    PixBuffer::Buf3(buf) => {
                        for xpos in 0..pixsize {
                            let offset = (xpos as usize) * 3;
                            buffer[offset] = (buf[xpos].r * 255.0) as u8;
                            buffer[offset+1] = (buf[xpos].g * 255.0) as u8;
                            buffer[offset+2] = (buf[xpos].b * 255.0) as u8;
                        }
                    }
                }
            })
        })?;
        canvas.copy(&texture, None, None)?;

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running Ok(())
                },
                Event::KeyDown { keycode: Some(Keycode::Space), .. } => {
                    pause = !pause;
                },
                _ => {}
            }
        }

        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
        
    }
}
