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

    sdl_context.mouse().set_relative_mouse_mode(true);

    let mut engine = Engine::new(&window).await;

    let mut event_pump = sdl_context.event_pump().unwrap();

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                Event::Window {
                    win_event: WindowEvent::Resized(width, height),
                    ..
                } => engine.resize((width as u32, height as u32)),
                Event::KeyDown {keycode: Some(Keycode::Escape),..} => {
                    sdl_context.mouse().set_relative_mouse_mode(!engine.relative_mouse);
                }
                _ => {}
            }
        }
        let scancodes = event_pump
            .keyboard_state()
            .pressed_scancodes()
            .collect::<Vec<_>>();
        let mouse_state = event_pump.relative_mouse_state();
        engine.input(&scancodes, &mouse_state);
        engine.update(&sdl_context);
        engine.frame_number += 1;
        match engine.render() {
            Ok(_) => {}
            Err(wgpu::SurfaceError::Lost) => engine.resize(engine.size),
            Err(wgpu::SurfaceError::OutOfMemory) => break 'running,
            Err(e) => eprintln!("{:?}", e),
        }
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}
