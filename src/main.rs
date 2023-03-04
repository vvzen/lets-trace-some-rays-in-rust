#![deny(clippy::all)]
#![forbid(unsafe_code)]

use std::sync::{Arc, Mutex};

use log::error;
use pixels::{wgpu, Error, PixelsBuilder, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

mod gui;
mod ltsr;

use crate::gui::constants::{
    RENDER_BUFFER_HEIGHT, RENDER_BUFFER_SIZE, RENDER_BUFFER_WIDTH, WINDOW_HEIGHT, WINDOW_WIDTH,
};
use crate::gui::Framework;

fn main() -> Result<(), Error> {
    env_logger::init();
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();
    let window = {
        let size = LogicalSize::new(WINDOW_WIDTH as f64, WINDOW_HEIGHT as f64);
        WindowBuilder::new()
            .with_title("Sample Framebuffer in Pixels + egui")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };

    // This will store the image that we draw, it's allocated on the heap
    // The mutex protects the data, the Arc helps share the memory across threads
    let render_buffer = Arc::new(Mutex::new(vec![1.0; RENDER_BUFFER_SIZE]));

    let (mut pixels, mut framework) = {
        let window_size = window.inner_size();
        let scale_factor = window.scale_factor() as f32;

        let surface_texture =
            SurfaceTexture::new(RENDER_BUFFER_WIDTH, RENDER_BUFFER_HEIGHT, &window);

        let pixels = PixelsBuilder::new(RENDER_BUFFER_WIDTH, RENDER_BUFFER_WIDTH, surface_texture)
            .texture_format(wgpu::TextureFormat::Rgba8UnormSrgb)
            .build()?;

        let framework = Framework::new(
            &event_loop,
            window_size.width,
            window_size.height,
            scale_factor,
            &pixels,
            render_buffer,
        );

        (pixels, framework)
    };

    event_loop.run(move |event, _, control_flow| {
        // Handle input events
        if input.update(&event) {
            // Close events
            if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                *control_flow = ControlFlow::Exit;
                return;
            }

            // Update the scale factor
            if let Some(scale_factor) = input.scale_factor() {
                framework.scale_factor(scale_factor);
            }

            // Resize the window
            if let Some(size) = input.window_resized() {
                if let Err(err) = pixels.resize_surface(size.width, size.height) {
                    error!("pixels.resize_surface() failed: {err}");
                    *control_flow = ControlFlow::Exit;
                    return;
                }
                framework.resize(size.width, size.height);
            }

            // Update internal state and request a redraw
            framework.app.update();
            window.request_redraw();
        }

        match event {
            Event::WindowEvent { event, .. } => {
                // Update egui inputs
                framework.handle_event(&event);
            }
            // Draw the current frame
            Event::RedrawRequested(_) => {
                // Draw the world
                framework.app.draw(pixels.get_frame_mut());

                // Prepare egui
                framework.prepare(&window);

                // Render everything together
                // TODO: I really don't want the texture to alway scale
                // up to the whole window, how can I achieve that?
                let render_result = pixels.render_with(|encoder, render_target, context| {
                    // Render the world texture
                    context.scaling_renderer.render(encoder, render_target);

                    // Render egui
                    framework.render(encoder, render_target, context);

                    Ok(())
                });

                // Basic error handling
                if let Err(err) = render_result {
                    error!("pixels.render() failed: {err}");
                    *control_flow = ControlFlow::Exit;
                }
            }
            _ => (),
        }
    });
}
