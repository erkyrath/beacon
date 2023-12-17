extern crate sdl2; 

use std::time::Duration;

use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;

fn main() -> Result<(), String> {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
 
    let window = video_subsystem.window("beacon demo", 800, 100)
        .position_centered()
        .build()
        .unwrap();
 
    let mut canvas = window.into_canvas().build().unwrap();
    let tc = canvas.texture_creator();
    let mut texture = tc.create_texture_streaming(PixelFormatEnum::RGB24, 160, 1).unwrap();
 
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();
    
    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut i: u32 = 0;
    'running: loop {
        i = i+1;
        texture.with_lock(None, |buffer: &mut [u8], _pitch: usize| {
            for xpos in 0..160 {
                let val = ((xpos as f32) + (i as f32) * 0.1).sin();
                let offset = (xpos as usize) * 3;
                buffer[offset] = (128.0 + val * 128.0).floor() as u8;
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
