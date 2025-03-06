#![deny(clippy::all)]
#![forbid(unsafe_code)]

use std::process::exit;

use systems::{CHIP8_DISP_HEIGHT, CHIP8_DISP_WIDTH};

//use std::hash::Hasher;
use {
    error_iter::ErrorIter as _,
    log::error,
    pixels::{Pixels, SurfaceTexture},
    std::{
        error::Error,
        sync::{Arc, RwLock},
        time::Duration,
    },
    winit::{
        dpi::LogicalSize,
        event::{Event, WindowEvent},
        event_loop::EventLoop,
        keyboard::KeyCode,
        window::WindowBuilder,
    },
    winit_input_helper::WinitInputHelper,
};

mod errors;
mod gui;
mod mem;
mod systems;
mod debug;
use crate::{
    gui::Framework,
    systems::{Chip8, System},
};

const SCALE: u32 = 16;
const WIN_WIDTH: u32 = CHIP8_DISP_WIDTH as u32 * SCALE;
const WIN_HEIGHT: u32 = CHIP8_DISP_HEIGHT as u32 * SCALE; // TODO: add egui toolbar height ?

fn open_bytes(path: &String) -> Result<Vec<u8>, Box<dyn Error>> {
    Ok(std::fs::read(path)?)
}

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    // rather use GUI techniques
    let args: Vec<String> = std::env::args().collect();
    let path = if let Some(arg) = args.get(1) {
        arg
    } else {
        println!("Usage : emu [CHIP-8 program]");
        exit(1);
    };
    // might use clap later instead to discern between systems and have some debug options

    let event_loop = EventLoop::new().unwrap();
    let input = Arc::new(RwLock::new(WinitInputHelper::new()));
    let input_shared = input.clone();
    let window = {
        let size = LogicalSize::new(WIN_WIDTH as f64, WIN_HEIGHT as f64);
        WindowBuilder::new()
            .with_title("Rusty Chip8")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };
    let window_shared = Arc::new(window);
    let pixels_window = window_shared.clone();

    let (pixels, mut framework) = {
        let window_size = window_shared.inner_size();
        let scale_factor = window_shared.scale_factor() as f32;
        let surface_texture =
            SurfaceTexture::new(window_size.width, window_size.height, pixels_window);
        let pixels = Pixels::new(CHIP8_DISP_WIDTH as u32, CHIP8_DISP_HEIGHT as u32, surface_texture)?;
        let framework = Framework::new(
            &event_loop,
            window_size.width,
            window_size.height,
            scale_factor,
            &pixels,
        );

        (pixels, framework)
    };

    let program_data = open_bytes(path)?;
    // TODO: clean error handling on rwlock thingies
    let chip8 = Arc::new(RwLock::new(Chip8::init()));
    let _ = chip8.write().unwrap().load_program(&program_data);
    let chip8_share = chip8.clone();

    // TODO: verify if it's really useful or not at the end
    let pixels_shared = Arc::new(RwLock::new(pixels));
    let pixels_thread = pixels_shared.clone();

    let _chip8_thread = std::thread::spawn(move || {
        loop {
            {
                let mut chip8 = chip8_share.write().unwrap();
                if let Err(e) = chip8.exec_instruction(input_shared.clone()) {
                    println!("{e}");
                    println!("{}", chip8);
                    println!("{}", chip8.get_mem());
                    println!("{}", chip8.get_backtrace());
                    return;
                }
            }
            std::thread::sleep(Duration::from_micros(2000));
        }
    });

    let res =
        event_loop.run(|event, elwt| {
            // Handle input events
            if input.write().unwrap().update(&event) {
                // Close events
                if input.read().unwrap().key_pressed(KeyCode::Escape)
                    || input.read().unwrap().close_requested()
                {
                    elwt.exit();
                    return;
                }

                // Update the scale factor
                if let Some(scale_factor) = input.read().unwrap().scale_factor() {
                    framework.scale_factor(scale_factor);
                }

                // Resize the window
                if let Some(size) = input.read().unwrap().window_resized() {
                    if let Err(err) = pixels_shared
                        .write()
                        .unwrap()
                        .resize_surface(size.width, size.height)
                    {
                        log_error("pixels.resize_surface", err);
                        elwt.exit();
                        return;
                    }
                    framework.resize(size.width, size.height);
                }

                // Update internal state and request a redraw
                // TODO: message cpu thread that this is a vblank ?
                // TODO: probe thread to see if it's alive or juste dead
                window_shared.request_redraw();
            }

            match event {
                // Draw the current frame
                Event::WindowEvent { event: WindowEvent::RedrawRequested, .. } => {
                    // Draw the world
                    chip8
                        .read()
                        .unwrap()
                        .set_pixels_frame(pixels_shared.write().unwrap().frame_mut());

                    // Prepare egui
                    framework.prepare(&window_shared);

                    // Render everything together
                    let render_result = pixels_shared.read().unwrap().render_with(
                        |encoder, render_target, context| {
                            // Render the world texture
                            context.scaling_renderer.render(encoder, render_target);

                            // Render egui
                            framework.render(encoder, render_target, context);

                            Ok(())
                        },
                    );

                    // Basic error handling
                    if let Err(err) = render_result {
                        log_error("pixels.render", err);
                        elwt.exit();
                    }
                }
                Event::WindowEvent { event, .. } => {
                    // Update egui inputs
                    framework.handle_event(&window_shared, &event);
                }
                _ => (),
            }
        });
    Ok(res?)
}

fn log_error<E: std::error::Error + 'static>(method_name: &str, err: E) {
    // TODO: verify
    // usefulness of
    // error_iter crate
    error!("{method_name}() failed: {err}");
    for source in err.sources().skip(1) {
        error!("  Caused by: {source}");
    }
}
