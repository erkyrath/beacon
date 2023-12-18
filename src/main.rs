#![allow(dead_code)]
#![allow(unused_imports)]

extern crate sdl2; 

use std::time::Duration;

use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;

mod pulser;
mod context;

fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
 
    let window = video_subsystem.window("beacon demo", 800, 100)
        .position_centered()
        .build()
        .map_err(|err| err.to_string())?;
 
    let mut canvas = window.into_canvas().build()
        .map_err(|err| err.to_string())?;
    let tc = canvas.texture_creator();
    let mut texture = tc.create_texture_streaming(PixelFormatEnum::RGB24, 160, 1)
        .map_err(|err| err.to_string())?;
 
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();
    
    let mut event_pump = sdl_context.event_pump()?;

    let mut ctx = context::RunContext::new();
    let mut pulser = pulser::Pulser::new(&ctx);
    let mut mainbuf: Vec<f32> = vec![0.0; 160];
        
    'running: loop {
        ctx.tick();

        pulser.tick(&ctx);
        pulser.render(&ctx, &mut mainbuf);
        
        texture.with_lock(None, |buffer: &mut [u8], _pitch: usize| {
            for xpos in 0..160 {
                //let val = ((xpos as f32) + (i as f32) * 0.1).sin();
                let offset = (xpos as usize) * 3;
                buffer[offset] = (mainbuf[xpos] * 255.0) as u8;
                buffer[offset+1] = 64;
                buffer[offset+2] = 0;
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
