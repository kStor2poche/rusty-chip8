use winit::keyboard::Key;

use {
    anyhow::Result,
    log::error,
    pixels::{Pixels, SurfaceTexture},
    std::{
        sync::{Arc, RwLock},
        time::Duration,
        process::exit,
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
mod disas;
use crate::{
    gui::Framework,
    systems::{Chip8, System, CHIP8_DISP_HEIGHT, CHIP8_DISP_WIDTH},
};

const SCALE: u32 = 16;
const WIN_WIDTH: u32 = CHIP8_DISP_WIDTH as u32 * SCALE;
const WIN_HEIGHT: u32 = CHIP8_DISP_HEIGHT as u32 * SCALE; // TODO: add egui toolbar height ?

fn open_bytes(path: &String) -> Result<Vec<u8>> {
    Ok(std::fs::read(path)?)
}

fn main() -> Result<()> {
    env_logger::init();

    // rather use GUI techniques
    let args: Vec<String> = std::env::args().collect();
    let path = if let Some(arg) = args.get(1) {
        arg
    } else {
        println!("Usage : emu [CHIP-8 program]");
        exit(1);
    };

    let event_loop = EventLoop::new()?;
    let input = Arc::new(RwLock::new(WinitInputHelper::new()));
    let input_shared = input.clone();
    let window = {
        let size = LogicalSize::new(WIN_WIDTH as f64, WIN_HEIGHT as f64);
        WindowBuilder::new()
            .with_title("Rusty Chip8")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)?
    };

    let (mut pixels, mut framework) = {
        let window_size = window.inner_size();
        let scale_factor = window.scale_factor() as f32;
        let surface_texture =
            SurfaceTexture::new(window_size.width, window_size.height, &window);
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
    let chip8 = Arc::new(RwLock::new(Chip8::init()));
    chip8.write().expect("Lock poisoned").load_program(&program_data)?;
    let chip8_share = chip8.clone();

    let chip8_thread = std::thread::spawn(move || {
        loop {
            let mut chip8 = chip8_share.write().expect("Lock poisoned");
            if let Err(e) = chip8.exec_instruction(input_shared.clone()) {
                println!("{e}");
                println!("{}", chip8.get_state());
                println!("{}", chip8.get_mem());
                println!("{}", chip8.get_backtrace());
                return;
            }
            drop(chip8);
            std::thread::sleep(Duration::from_micros(200));
        }
    });

    let res =
        event_loop.run(|event, elwt| {
            // Handle input events
            if input.write().expect("Lock poisoned").update(&event) {
                // Close events
                if input.read().expect("Lock poisoned").key_pressed(KeyCode::Escape)
                    || input.read().expect("Lock poisoned").key_pressed_logical(Key::Character("q"))
                    || input.read().expect("Lock poisoned").close_requested()
                {
                    elwt.exit();
                    return;
                }

                // Update the scale factor
                // TODO: see how to not crash from scaling with egui ^^
                if let Some(scale_factor) = input.read().expect("Lock poisoned").scale_factor() {
                    framework.scale_factor(scale_factor);
                }

                // Resize the window
                if let Some(size) = input.read().expect("Lock poisoned").window_resized() {
                    if let Err(err) = pixels.resize_surface(size.width, size.height) {
                        error!("On surface resize, {}", err);
                        elwt.exit();
                        return;
                    }
                    framework.resize(size.width, size.height);
                }

                // Update internal state and request a redraw
                // TODO: message cpu thread that this is a vblank ?
                if chip8_thread.is_finished() {
                    // TODO: gui things
                }
                window.request_redraw();
            }

            match event {
                // Draw the current frame
                Event::WindowEvent { event: WindowEvent::RedrawRequested, .. } => {
                    // Draw the world
                    chip8
                        .read()
                        .expect("Lock poisoned")
                        .set_pixels_frame(pixels.frame_mut());

                    // Prepare egui
                    framework.prepare(&window);

                    // Render everything together
                    let render_result = pixels.render_with(
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
                        error!("on render_result: {}", err);
                        elwt.exit();
                    }
                }
                Event::WindowEvent { event, .. } => {
                    // Update egui inputs
                    framework.handle_event(&window, &event);
                }
                _ => (),
            }
        });
    Ok(res?)
}
