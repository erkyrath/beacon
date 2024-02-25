#![allow(dead_code)]
#![allow(unused_imports)]

use gumdrop::Options;

#[cfg(feature = "sdl2")]
extern crate sdl2; 

#[macro_use]
extern crate lazy_static;

use std::fs::File;
use std::io::BufWriter;
use std::time::Instant;
use std::time::Duration;
use std::time::SystemTime;

mod pixel;
mod lerp;
mod op;
mod script;
mod parse;
mod param;
mod clock;
mod runner;
mod context;
mod waves;
mod pulser;

use script::{Script, ScriptIndex};
use runner::{Runner, RunContext, RunContextWrap, PixBuffer};
use context::scriptcontext::{ScriptRunner, ScriptContext};

#[derive(Options, Debug)]
pub struct AppOptions {
    #[options(free)]
    args: Vec<String>,
    
    #[options(help = "print help message")]
    help: bool,

    #[options(long="size", help = "pixel count (default 160)")]
    size: Option<usize>,

    #[options(long="fps", help = "frames per second (default 60)")]
    fps: Option<u32>,

    #[options(long="dump", help = "dump script to stdout")]
    dump: bool,

    #[options(long="file", help = "run script headless and write to a file (\"file%.png\")")]
    writefile: Option<String>,

    #[options(long="led", help = "display on connected LEDs")]
    led: bool,

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

    #[options(long="count", help = "frame count (for --file)")]
    framecount: Option<usize>,

    #[options(long="skip", help = "frame skip (for --file)")]
    frameskip: Option<usize>,

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
        return;
    }
    
    let runner = ScriptRunner::new(script);
    let fps = opts.fps.unwrap_or(60);

    if let Some(filename) = &opts.writefile {
        let framecount = opts.framecount.unwrap_or(16);
        let frameskip = opts.frameskip.unwrap_or(0);
        let pixheight = opts.winheight.unwrap_or(4) as usize;
        let res = run_writefile(filename, &runner, pixsize, pixheight, fps, framecount, frameskip);
        match res {
            Err(msg) => {
                println!("{msg}");
            },
            Ok(()) => {
                println!("wrote {} images", framecount);
            },
        }
    }
    else if opts.spin {
        let dur: f64 = 0.1;
        let res = run_spin(&runner, pixsize, fps, dur);
        match res {
            Err(msg) => {
                println!("{msg}");
            },
            Ok(count) => {
                println!("{} frames in {} seconds", count, dur);
            },
        }
    }
    else if opts.led {
        let res = run_leds(&runner, pixsize, fps);
        if let Err(msg) = res {
            println!("{msg}");
        }
    }
    else {
        let winwidth = opts.winwidth.unwrap_or(800);
        let winheight = opts.winheight.unwrap_or(100);
        let res = run_sdl(&runner, pixsize, fps, filename, opts.watchfile, opts.showpower, winwidth, winheight);
        if let Err(msg) = res {
            println!("{msg}");
        }
    }
}

fn run_spin(runner: &dyn Runner, pixsize: usize, fps: u32, seconds: f64) -> Result<usize, String> {
    let mut ctx = runner.build(pixsize, Some(fps));
    let mut count = 0;
    let start = Instant::now();
    
    loop {
        ctx.tick();
        count += 1;
        let dur = start.elapsed();
        if dur.as_secs_f64() > seconds {
            break;
        }
        if ctx.done() {
            return Err("script ended before time".to_string());
        }
    }
    
    Ok(count)
}

#[cfg(not(feature = "png"))]
fn run_writefile(_filename: &str, _runner: &dyn Runner, _pixsize: usize, _pixheight: usize, _fps: u32, _framecount: usize, _frameskip: usize) -> Result<(), String> {
    return Err("png feature not available".to_string());
}

#[cfg(feature = "png")]
fn run_writefile(filename: &str, runner: &dyn Runner, pixsize: usize, pixheight: usize, fps: u32, framecount: usize, frameskip: usize) -> Result<(), String> {
    let mut ctx = runner.build(pixsize, Some(fps));

    for _ in 0..frameskip {
        ctx.tick();
    }

    for count in 0..framecount {
        ctx.tick();
        if ctx.done() {
            break;
        }
        
        let mut buffer: Vec<u8> = vec![0; 4*pixsize*pixheight];
        ctx.applybuf(|pixbuf| {
            match pixbuf {
                PixBuffer::Buf1(buf) => {
                    for ypos in 0..pixheight {
                        for xpos in 0..pixsize {
                            let offset = ypos * pixsize * 4 + xpos * 4;
                            buffer[offset] = (buf[xpos] * 255.0) as u8;
                            buffer[offset+1] = buffer[offset];
                            buffer[offset+2] = buffer[offset];
                            buffer[offset+3] = 255;
                        }
                    }
                },
                PixBuffer::Buf3(buf) => {
                    for ypos in 0..pixheight {
                        for xpos in 0..pixsize {
                            let offset = ypos * pixsize * 4 + xpos * 4;
                            buffer[offset] = (buf[xpos].r * 255.0) as u8;
                            buffer[offset+1] = (buf[xpos].g * 255.0) as u8;
                            buffer[offset+2] = (buf[xpos].b * 255.0) as u8;
                            buffer[offset+3] = 255;
                        }
                    }
                }
            }
        });
        
        let tempfile = filename.replace("%", &format!("{:04}", count).to_string());
        let file = File::create(tempfile)
            .map_err(|err| err.to_string())?;
        let ref mut fwriter = BufWriter::new(file);
        let mut encoder = png::Encoder::new(fwriter, pixsize as u32, pixheight as u32);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().unwrap();
        writer.write_image_data(&buffer)
            .map_err(|err| err.to_string())?;
    }

    Ok(())
}

#[cfg(not(feature = "rpi"))]
fn run_leds(_runner: &dyn Runner, _pixsize: usize, _fps: u32) -> Result<(), String> {
    return Err("rpi feature not available".to_string());
}

#[cfg(feature = "rpi")]
fn run_leds(runner: &dyn Runner, pixsize: usize, fps: u32) -> Result<(), String> {
    use rppal::spi::{Bus, SlaveSelect, Spi};
    use smart_leds_trait::{RGB8, SmartLedsWrite};

    let ticktime = 1_000_000_000u32 / fps;
    
    //###
    let info = rppal::system::DeviceInfo::new()
        .map_err(|err| err.to_string())?;
    println!("### info {:?}", info);

    let spi = Spi::new(
        Bus::Spi0,
        SlaveSelect::Ss0,
        20_000_000, // Hz?
        // Mode0: IdleLow (CPOL0), CaptureOnFirstTransition (CPHA0)
        rppal::spi::Mode::Mode0)
        .map_err(|err| err.to_string())?;
    println!("### spi {:?}", spi);

    let mut driver = apa102_spi::Apa102::new(spi);
    //### might need to change default BGR

    let mut ctx = runner.build(pixsize, None);
    
    loop {
        ctx.tick();

        let mut buffer: Vec<RGB8> = vec![RGB8::default(); pixsize];
        
        ctx.applybuf(|pixbuf| {
            match pixbuf {
                PixBuffer::Buf1(buf) => {
                    for xpos in 0..pixsize {
                        buffer[xpos] = RGB8::new((buf[xpos] * 255.0) as u8, (buf[xpos] * 255.0) as u8, (buf[xpos] * 255.0) as u8);
                    }
                },
                PixBuffer::Buf3(buf) => {
                    for xpos in 0..pixsize {
                        buffer[xpos] = RGB8::new((buf[xpos].r * 255.0) as u8, (buf[xpos].g * 255.0) as u8, (buf[xpos].b * 255.0) as u8);
                    }
                },
            };
        });

        //### apply gamma and brightness limiter?
        driver.write(buffer)
            .map_err(|err| err.to_string())?;

        if ctx.done() {
            break;
        }
        
        ::std::thread::sleep(Duration::new(0, ticktime));
    }

    driver.free();
    Ok(())
}

#[cfg(not(feature = "sdl2"))]
fn run_sdl(_runner: &dyn Runner, _pixsize: usize, _fps: u32, _filename: &str, _watchfile: bool, _showpower: bool, _winwidth: u32, _winheight: u32) -> Result<(), String> {
    return Err("sdl2 feature not available".to_string());
}

#[cfg(feature = "sdl2")]
fn run_sdl(runner: &dyn Runner, pixsize: usize, fps: u32, filename: &str, watchfile: bool, showpower: bool, winwidth: u32, winheight: u32) -> Result<(), String> {
    use sdl2::pixels::Color;
    use sdl2::event::Event;
    use sdl2::keyboard::Keycode;
    use sdl2::pixels::PixelFormatEnum;
    
    let ticktime = 1_000_000_000u32 / fps;
    
    let margin: u32 = 16;
    let copyrect = sdl2::rect::Rect::new(0, margin as i32, winwidth, winheight);
    
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

    let wintitle = format!("beacon: {} ({}p)", filename, pixsize);
    let window = video_subsystem.window(wintitle.as_str(), winwidth, winheight+2*margin)
        .position_centered()
        .build()
        .map_err(|err| err.to_string())?;

    let mut canvas = window.into_canvas()
        .present_vsync()
        .build()
        .map_err(|err| err.to_string())?;
    let tc = canvas.texture_creator();
    let mut texture = tc.create_texture_streaming(PixelFormatEnum::RGB24, pixsize as u32, 1)
        .map_err(|err| err.to_string())?;
 
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();
    
    let mut event_pump = sdl_context.event_pump()?;

    let mut ctx = runner.build(pixsize, None);
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
                        let newrunner = ScriptRunner::new(newscript);
                        ctx = newrunner.build(pixsize, None);
                        powertime = 0.0;
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
                // Really the off pixels require a bit of power, maybe 5%?
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
        canvas.clear();
        canvas.copy(&texture, None, Some(copyrect))?;

        if ctx.done() {
            break Ok(())
        }

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
        ::std::thread::sleep(Duration::new(0, ticktime));
        
    }
}
