use std::sync::{Arc, Mutex};

use colstodian::spaces::{AcesCg, EncodedSrgb};
use colstodian::tonemap::{PerceptualTonemapper, PerceptualTonemapperParams, Tonemapper};
use colstodian::{Color, Display};

use crate::gui::image::render_scene;

/// Representation of the application state
pub struct ApplicationState {
    // RGB 32 bit
    // pub framebuffer: &'s Vec<f32>,
    pub framebuffer: Arc<Mutex<Vec<f32>>>,
}

impl ApplicationState {
    /// Create a new `ApplicationState` instance that can draw a moving box.
    pub fn new(framebuffer: Arc<Mutex<Vec<f32>>>) -> Self {
        // eprintln!("Size of render buffer: {}", render_buffer.len());

        let mut buffer_data = &mut framebuffer.lock().unwrap();

        // TODO: This should happen in another thread
        eprintln!("Started rendering..");
        render_scene(&mut buffer_data);
        eprintln!("finished rendering..");

        let fb_p = framebuffer.clone();

        Self { framebuffer: fb_p }
    }

    /// Update the Application internal state
    pub fn update(&mut self) {
        // TODO: here goes any update logic
    }

    // Draw to the frame buffer
    // Assumes the default texture format: `wgpu::TextureFormat::Rgba8UnormSrgb`
    // This means:
    //     Red, green, blue, and alpha channels.
    //     8 bit integer per channel.
    //     Srgb-color [0, 255] converted to/from linear-color float [0, 1] in shader
    // See more formats here: https://docs.rs/wgpu/latest/wgpu/enum.TextureFormat.html
    pub fn draw(&self, frame: &mut [u8]) {
        // TODO: try_lock
        let buffer_data = self.framebuffer.lock().unwrap();

        let it = std::iter::zip(frame.chunks_exact_mut(4), buffer_data.chunks_exact(4));
        for (_, (pixel, render_pixel)) in it.enumerate() {
            // Here we draw the pixels!
            // In my case, I already drew them, so I can copy them around
            // and the bits of math to convert from scene referred to display referred

            // Recreate the Scene Linear color struct that we know we used
            // For the sake of simplicity and saving memory, our array is composed of f32
            // instead of propert color structs. Here we recreate the colstodian color struct
            // on the fly so we can do the conversion to 8bit sRGB
            let rendered_color =
                colstodian::color::acescg(render_pixel[0], render_pixel[1], render_pixel[2]);
            let alpha = render_pixel[3];

            // Use a standard Tonemap to go from ACEScg HDR to SDR
            let params = PerceptualTonemapperParams::default();
            let tonemapped: Color<AcesCg, Display> =
                PerceptualTonemapper::tonemap(rendered_color, params).convert();

            // Encode in sRGB so we're ready to display or write to an image
            let encoded = tonemapped.convert::<EncodedSrgb>();

            // Convert to 8bit
            let rgb: [u8; 3] = encoded.to_u8();

            // Can I avoid doing a copy here ?
            let rgba: [u8; 4] = [rgb[0], rgb[1], rgb[2], (255 as f32 * alpha) as u8];

            pixel.copy_from_slice(&rgba);
        }
    }
}
