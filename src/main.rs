#![deny(clippy::all)]
#![forbid(unsafe_code)]

use std::error::Error;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use std::hash::Hasher;

use crate::gui::Framework;
use error_iter::ErrorIter as _;
use log::error;
use pixels::{Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::EventLoop;
use winit::keyboard::KeyCode;
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

mod gui;
mod systems;
mod errors;
mod mem;

const WIDTH: u32 = 640;
const HEIGHT: u32 = 480;
const BOX_SIZE: i16 = 64;

/// Representation of the application state. In this example, a box will bounce around the screen.
#[derive(Debug)]
struct World {
    box_x: i16,
    box_y: i16,
    velocity_x: i16,
    velocity_y: i16,
}

fn open_bytes(path: &String) -> Result<Vec<u8>, Box<dyn Error>> {
    Ok(std::fs::read(path)?)
}

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    // rather use GUI techniques
    let args: Vec<String> = std::env::args().collect();
    let path = args.get(1).expect("Usage : emu [CHIP-8 program]");
    // might use clap later instead to discern between systems and have some debug options

    let program_data = open_bytes(path)?;

    let event_loop = EventLoop::new().unwrap();
    let mut input = WinitInputHelper::new();
    let window = {
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        WindowBuilder::new()
            .with_title("Hello Pixels + egui")
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
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, pixels_window);
        let pixels = Pixels::new(WIDTH, HEIGHT, surface_texture)?;
        let framework = Framework::new(
            &event_loop,
            window_size.width,
            window_size.height,
            scale_factor,
            &pixels,
        );

        (pixels, framework)
    };

    let world = Arc::new(RwLock::new(World::new())); // TODO: create cpu thread
    let world_shared = world.clone();
    fn world_loop(world: &mut World, pixels: Arc<RwLock<Pixels>>) {
        loop {
            world.update();
        }
    }
    let pixels_shared = Arc::new(RwLock::new(pixels));
    let pixels_thread = pixels_shared.clone();
    let world_thread = std::thread::spawn(move || {
        loop {
            {world_shared.write().unwrap().update()};
            drop(world_shared.read().inspect(|w| {
                //println!("{:?}", w);
            }));
            //std::thread::sleep(Duration::from_millis(1));
        }
    });

    let res = event_loop.run(|event, elwt| {
        // Handle input events
        if input.update(&event) {
            // Close events
            if input.key_pressed(KeyCode::Escape) || input.close_requested() {
                elwt.exit();
                return;
            }

            // Update the scale factor
            if let Some(scale_factor) = input.scale_factor() {
                framework.scale_factor(scale_factor);
            }

            // Resize the window
            if let Some(size) = input.window_resized() {
                if let Err(err) = pixels_shared.write().unwrap().resize_surface(size.width, size.height) {
                    log_error("pixels.resize_surface", err);
                    elwt.exit();
                    return;
                }
                framework.resize(size.width, size.height);
            }

            // Update internal state and request a redraw
            // TODO: message thread to get update on buffer and 
            window_shared.request_redraw();
        }

        match event {
            // Draw the current frame
            Event::WindowEvent {
                event: WindowEvent::RedrawRequested,
                ..
            } => {
                // Draw the world
                world.read().unwrap().draw(pixels_shared.write().unwrap().frame_mut());

                // Prepare egui
                framework.prepare(&window_shared);

                // Render everything together
                let render_result = pixels_shared.read().unwrap().render_with(|encoder, render_target, context| {
                    // Render the world texture
                    context.scaling_renderer.render(encoder, render_target);

                    // Render egui
                    framework.render(encoder, render_target, context);

                    Ok(())
                });

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

fn log_error<E: std::error::Error + 'static>(method_name: &str, err: E) { // TODO: verify
                                                                          // usefulness of
                                                                          // error_iter crate
    error!("{method_name}() failed: {err}");
    for source in err.sources().skip(1) {
        error!("  Caused by: {source}");
    }
}

impl World {
    // Create a new `World` instance that can draw a moving box.
    fn new() -> Self {
        Self {
            box_x: 24,
            box_y: 16,
            velocity_x: 1,
            velocity_y: 1,
        }
    }

    // Update the `World` internal state; bounce the box around the screen.
    fn update(&mut self) {
        if self.box_x <= 0 || self.box_x + BOX_SIZE > WIDTH as i16 {
            self.velocity_x *= -1;
        }
        if self.box_y <= 0 || self.box_y + BOX_SIZE > HEIGHT as i16 {
            self.velocity_y *= -1;
        }

        self.box_x += self.velocity_x;
        self.box_y += self.velocity_y;
    }

    // Draw the `World` state to the frame buffer.
    //
    // Assumes the default texture format: `wgpu::TextureFormat::Rgba8UnormSrgb`
    fn draw(&self, frame: &mut [u8]) {
        let mut hasher = std::hash::DefaultHasher::new();
        std::hash::Hash::hash_slice(frame, &mut hasher);
        //println!("cur frame hash : {:x}", hasher.finish());
        //println!("(self is {self:?})");
        for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
            let x = (i % WIDTH as usize) as i16;
            let y = (i / WIDTH as usize) as i16;

            let inside_the_box = x >= self.box_x
                && x < self.box_x + BOX_SIZE
                && y >= self.box_y
                && y < self.box_y + BOX_SIZE;

            let rgba = if inside_the_box {
                [0x5e, 0x48, 0xe8, 0xff]
            } else {
                [0x48, 0xb2, 0xe8, 0xff]
            };

            pixel.copy_from_slice(&rgba);
        }
        let mut hasher = std::hash::DefaultHasher::new();
        std::hash::Hash::hash_slice(frame, &mut hasher);
        //println!("next frame hash : {:x}", hasher.finish());
    }
}
