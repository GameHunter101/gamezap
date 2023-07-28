use std::time::Duration;

use sdl2::{
    event::{Event, WindowEvent},
    keyboard::Keycode,
};

use crate::engine::Engine;

pub async fn run() {
    env_logger::init();

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("gameZap", 800, 600)
        .position_centered()
        .resizable()
        .vulkan()
        .build()
        .unwrap();

    let mut engine = Engine::new(&window).await;

    // canvas.set_draw_color(sdl2::pixels::Color::RGB(100, 100, 100));
    // canvas.clear()
    // canvas.present();

    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        for event in event_pump.poll_iter() {
            if !engine.input(&event) {
                match event {
                    Event::Quit { .. }
                    | Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    } => break 'running,
                    Event::Window {
                        win_event: WindowEvent::Resized(width, height),
                        ..
                    } => engine.resize((width as u32, height as u32)),
                    _ => {}
                }
            }
            engine.update();
            match engine.render() {
                Ok(_) => {}
                Err(wgpu::SurfaceError::Lost) => engine.resize(engine.size),
                Err(wgpu::SurfaceError::OutOfMemory) => break 'running,
                Err(e) => eprintln!("{:?}", e),
            }
            ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
        }
    }
}
