use std::sync::{Arc, Mutex, RwLock};
use std::thread;

use colstodian::spaces::{AcesCg, EncodedSrgb};
use colstodian::tonemap::{PerceptualTonemapper, PerceptualTonemapperParams, Tonemapper};
use colstodian::{Color, Display};

use crate::gui::constants::RENDER_BUFFER_SIZE;
use crate::gui::image::render_scene;

/// Representation of the application state
pub struct ApplicationState {
    // RGBA 32 bit
    pub render_buffer: Arc<Mutex<Vec<f32>>>,
    // RGBA 8 bit
    pub pixels_data: Vec<u8>,
}

impl ApplicationState {
    /// Create a new `ApplicationState` instance that can do the rendering

    pub fn new(render_buffer: Arc<Mutex<Vec<f32>>>) -> Self {
        let buffer = render_buffer.clone();

        let buffer_data = render_buffer.lock().unwrap();
        let mut pixels_data: Vec<u8> = vec![0; RENDER_BUFFER_SIZE];

        let it = std::iter::zip(buffer_data.chunks_exact(4), pixels_data.chunks_exact_mut(4));

        for (_, (f32_pixel, u8_pixel)) in it.enumerate() {
            // For the sake of simplicity and saving memory, our array is composed of f32
            // instead of propert color structs. Here we recreate the colstodian color struct
            // on the fly so we can do the conversion to 8bit sRGB and go to display referred
            // by applying default a SDR tone mapping
            let rendered_color =
                colstodian::color::acescg(f32_pixel[0], f32_pixel[1], f32_pixel[2]);

            // Use a standard Tonemap to go from ACEScg HDR to SDR
            let params = PerceptualTonemapperParams::default();
            let tonemapped: Color<AcesCg, Display> =
                PerceptualTonemapper::tonemap(rendered_color, params).convert();

            // Encode in sRGB so we're ready to display or write to an image
            let encoded = tonemapped.convert::<EncodedSrgb>();

            // Convert to 8bit
            let rgb: [u8; 3] = encoded.to_u8();
            let alpha = f32_pixel[3];

            // Can I avoid doing a copy here ?
            let rgba: [u8; 4] = [rgb[0], rgb[1], rgb[2], (255 as f32 * alpha) as u8];

            u8_pixel.copy_from_slice(&rgba);
        }

        Self {
            render_buffer: buffer,
            pixels_data,
        }
    }

    /// Update the Application internal state
    pub fn update(&mut self) {
        // Clone the pointer
        // TODO: try_lock
        // Spawn a new thread to do the rendering
        // let should_render = self.should_render.read().unwrap();

        // if *should_render {
        //     // *should_render = false;

        //     let handle = thread::spawn(move || {
        //         eprintln!("Started rendering..");
        //         let mut buffer_data = fb_p.lock().unwrap();

        //         render_scene(&mut buffer_data);
        //         eprintln!("finished rendering..");
        //     });

        //     handle.join().unwrap();
        // }
    }

    // Draw to the frame buffer
    // Assumes the default texture format: `wgpu::TextureFormat::Rgba8UnormSrgb`
    // This means:
    //     Red, green, blue, and alpha channels.
    //     8 bit integer per channel.
    //     Srgb-color [0, 255] converted to/from linear-color float [0, 1] in shader
    // See more formats here: https://docs.rs/wgpu/latest/wgpu/enum.TextureFormat.html
    pub fn draw(&self, frame: &mut [u8]) {
        // Here we draw the pixels!
        // In my case, I already drew them, so I can copy them around
        // and the bits of math to convert from scene referred to display referred
        // TODO: Only do this if stuff has changed
        frame.copy_from_slice(self.pixels_data.as_slice());

        // let it = std::iter::zip(frame.chunks_exact_mut(4), self.pixels_data.chunks_exact(4));
        // for (_, (pixel, render_pixel)) in it.enumerate() {
        //     // Here we draw the pixels!
        //     // In my case, I already drew them, so I can copy them around
        //     // and the bits of math to convert from scene referred to display referred

        //     // Recreate the Scene Linear color struct that we know we used
        //     // For the sake of simplicity and saving memory, our array is composed of f32
        //     // instead of propert color structs. Here we recreate the colstodian color struct
        //     // on the fly so we can do the conversion to 8bit sRGB
        //     let rendered_color =
        //         colstodian::color::acescg(render_pixel[0], render_pixel[1], render_pixel[2]);
        //     let alpha = render_pixel[3];

        //     // Use a standard Tonemap to go from ACEScg HDR to SDR
        //     let params = PerceptualTonemapperParams::default();
        //     let tonemapped: Color<AcesCg, Display> =
        //         PerceptualTonemapper::tonemap(rendered_color, params).convert();

        //     // Encode in sRGB so we're ready to display or write to an image
        //     let encoded = tonemapped.convert::<EncodedSrgb>();

        //     // Convert to 8bit
        //     let rgb: [u8; 3] = encoded.to_u8();

        //     // Can I avoid doing a copy here ?
        //     let rgba: [u8; 4] = [rgb[0], rgb[1], rgb[2], (255 as f32 * alpha) as u8];

        //     pixel.copy_from_slice(&rgba);
        // }
    }
}
