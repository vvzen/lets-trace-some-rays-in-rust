use std::boxed::Box;
use std::sync::Arc;
use std::time::Instant;

use anyhow;
use exr::prelude::{AnyChannel, AnyChannels, Encoding, FlatSamples, Image, Layer, LayerAttributes};
use glam::Vec3;
use rand::Rng;
use smallvec::smallvec;

// Color
use colstodian::spaces::{AcesCg, EncodedSrgb};
use colstodian::tonemap::{PerceptualTonemapper, PerceptualTonemapperParams, Tonemapper};
use colstodian::{color, Color, Display};

use crate::app::AppError;
use crate::constants::{
    NUM_SAMPLES_PER_PIXEL, RENDER_BUFFER_HEIGHT, RENDER_BUFFER_SIZE, RENDER_BUFFER_WIDTH,
};
use crate::ltsr::materials::{Lambertian, Metallic};
use crate::ltsr::{fit_range, ray_color, Camera, Scene, Sphere};

pub type SimpleOpenEXRImage = Image<Layer<AnyChannels<FlatSamples>>>;

#[derive(Debug, Clone)]
pub struct RenderTask {}

/// Sample function demostrating how to render a custom image
pub fn render_bg_image() -> Vec<f32> {
    let mut render_buffer = vec![0.0; RENDER_BUFFER_SIZE];

    // Render a in linear color space
    let mut index: usize = 0;
    for y in (0..RENDER_BUFFER_HEIGHT).rev() {
        for x in 0..RENDER_BUFFER_WIDTH {
            // Get normalized U,V coordinates as we move through the image
            let u = fit_range(x as f32, 0.0, RENDER_BUFFER_WIDTH as f32, 0.0, 1.0);
            let v = fit_range(y as f32, 0.0, RENDER_BUFFER_HEIGHT as f32, 0.0, 1.0);

            // Generate a gradient between two colors in AcesCG
            // TODO: Could we do this in LAB, and then convert to ACES CG ?
            let red = color::acescg::<colstodian::Scene>(1.0, 0.0, 0.0);
            let green = color::acescg::<colstodian::Scene>(0.0, 1.0, 0.0);
            let blue = color::acescg::<colstodian::Scene>(0.0, 0.0, 1.0);
            let h_blended = red.blend(green, u);
            let v_blended = red.blend(blue, v);
            let final_color = h_blended.blend(v_blended, 0.5);

            let rendered_color =
                color::acescg::<colstodian::Scene>(final_color.r, final_color.g, final_color.b);

            // R, G, B, A
            render_buffer[index + 0] = rendered_color.r;
            render_buffer[index + 1] = rendered_color.g;
            render_buffer[index + 2] = rendered_color.b;
            render_buffer[index + 3] = 1.0;

            index += 4;
        }
    }

    render_buffer.clone()
}

impl RenderTask {
    /// Sample function performing the rendering of basic 3D scene
    pub async fn render_scene() -> Result<Vec<f32>, AppError> {
        let mut render_buffer = vec![0.0; RENDER_BUFFER_SIZE];

        eprintln!(
            "Started rendering.. using {} rays per pixel",
            NUM_SAMPLES_PER_PIXEL
        );
        let start_time = Instant::now();

        // Shorthands
        let image_width = RENDER_BUFFER_WIDTH as f32;
        let image_height = RENDER_BUFFER_HEIGHT as f32;
        let aspect_ratio: f32 = image_width / image_height;

        // Camera properties
        let viewport_height = 2.0;
        let viewport_width = aspect_ratio * viewport_height;
        let camera = Camera::new(1.0, viewport_width, viewport_height);

        // Scene properties
        let mut scene = Scene::new();

        // Materials
        let col_ground = Vec3::new(0.8, 0.8, 0.1);
        let col_center = Vec3::new(0.7, 0.3, 0.3);
        let col_left = Vec3::new(0.8, 0.8, 0.8);
        let col_right = Vec3::new(0.8, 0.6, 0.2);

        let mat_ground = Arc::new(Lambertian::new(col_ground.clone()));
        let mat_center = Arc::new(Lambertian::new(col_center.clone()));
        let mat_left = Arc::new(Metallic::new(col_left.clone(), 0.3));
        let mat_right = Arc::new(Metallic::new(col_right.clone(), 1.0));

        // Let's check if our ray intersects some spheres
        let spheres_z = -1.0;
        let spheres_z = -1.0;
        let sphere_1 = Sphere::new(100.0, Vec3::new(0.0, -100.5, spheres_z), mat_ground.clone());
        let sphere_2 = Sphere::new(0.5, Vec3::new(0.0, 0.0, spheres_z), mat_center.clone());
        let sphere_3 = Sphere::new(0.5, Vec3::new(-1.0, 0.0, spheres_z), mat_left.clone());
        let sphere_4 = Sphere::new(0.5, Vec3::new(1.0, 0.0, spheres_z), mat_right.clone());

        scene.add_hittable(Box::new(sphere_1));
        scene.add_hittable(Box::new(sphere_2));
        scene.add_hittable(Box::new(sphere_3));
        scene.add_hittable(Box::new(sphere_4));

        // Sampling
        let mut rng = rand::thread_rng();

        // Ray bounces
        const MAX_DEPTH: i32 = 5;

        // Generate the image
        let mut index: usize = 0;
        for y in (0..RENDER_BUFFER_HEIGHT).rev() {
            for x in 0..RENDER_BUFFER_WIDTH {
                let mut pixel_color = Vec3::new(0.0, 0.0, 0.0);

                // Antialiasing: multiple samples per pixel
                for _ in 0..NUM_SAMPLES_PER_PIXEL {
                    // Get normalized U,V coordinates as we move through the image
                    let u = fit_range(x as f32 + rng.gen::<f32>(), 0.0, image_width, 0.0, 1.0);
                    let v = fit_range(y as f32 + rng.gen::<f32>(), 0.0, image_height, 0.0, 1.0);

                    // Aim the camera based on the current u,v coordinates
                    let ray = camera.get_ray_at_coords(u, v);
                    pixel_color += ray_color(&ray, &scene, MAX_DEPTH);
                }
                // Divide by the num of samples to get the average
                let scale = 1.0 / NUM_SAMPLES_PER_PIXEL as f32;
                pixel_color *= scale;

                // Convert from display-referred (0..1) to scene-referred (0..infinity)
                // TODO: Do the propert state conversion from Display to Scene
                let rendered_color =
                    color::acescg::<colstodian::Scene>(pixel_color.x, pixel_color.y, pixel_color.z);

                // R, G, B, A
                render_buffer[index + 0] = rendered_color.r;
                render_buffer[index + 1] = rendered_color.g;
                render_buffer[index + 2] = rendered_color.b;
                render_buffer[index + 3] = 1.0;

                index += 4;
            }
        }

        eprintln!("Finished rendering!");

        let elapsed_time = start_time.elapsed();
        eprintln!(
            "Elapsed time: {:?} (~{} seconds)",
            elapsed_time,
            elapsed_time.as_secs()
        );

        Ok(render_buffer)
    }

    /// Takes the floating point pixels from ``render_buffer`` and performs the
    /// math to store them in ``display_buffer``, ready to be presented as 8 bit
    /// bytes in the GUI. If ``is_data_pass`` is true, no color management will
    /// be applied.
    pub async fn convert_to_display_buffer(
        render_buffer: Vec<f32>,
        is_data_pass: bool,
    ) -> Result<Vec<u8>, AppError> {
        eprintln!("Converting from ACESCG linear to Display Color Space");
        let start_time = Instant::now();

        // Create a new buffer to host the converted pixels
        let mut display_buffer: Vec<u8> = vec![0; RENDER_BUFFER_SIZE];

        // Do the scene linear to display conversion
        let it = std::iter::zip(
            render_buffer.chunks_exact(4),
            display_buffer.chunks_exact_mut(4),
        );

        for (f32_pixel, u8_pixel) in it {
            // If we're working with a utility pass (eg: normals, etc.)
            // we only need to go from the 0-1 range to the 0-255 range
            // without going through tonemapping at all
            if is_data_pass {
                let rgba = [
                    (f32_pixel[0] * 255.0) as u8,
                    (f32_pixel[1] * 255.0) as u8,
                    (f32_pixel[2] * 255.0) as u8,
                    (f32_pixel[3] * 255.0) as u8,
                ];

                u8_pixel.copy_from_slice(&rgba);
                continue;
            }

            // For the sake of simplicity and saving memory, our array is composed of f32
            // instead of colostodian Color structs. Here we recreate the colstodian struct
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

        let elapsed_time = start_time.elapsed();
        eprintln!(
            "Conversion finished, elapsed time: {:?}, (~{} seconds)",
            elapsed_time,
            elapsed_time.as_secs()
        );

        Ok(display_buffer)
    }
}

pub fn convert_to_openexr(
    width: usize,
    height: usize,
    render_buffer: &Vec<f32>,
) -> anyhow::Result<SimpleOpenEXRImage> {
    let resolution = (width, height);

    // A vec for each channel
    let mut r_vec: Vec<f32> = Vec::new();
    let mut g_vec: Vec<f32> = Vec::new();
    let mut b_vec: Vec<f32> = Vec::new();

    // Fill in the RGB channels
    for f32_color in render_buffer.chunks_exact(4) {
        r_vec.push(f32_color[0]);
        g_vec.push(f32_color[1]);
        b_vec.push(f32_color[2]);
    }

    // Save the data into the channels
    let r_channel = AnyChannel::new("R", FlatSamples::F32(r_vec));
    let g_channel = AnyChannel::new("G", FlatSamples::F32(g_vec));
    let b_channel = AnyChannel::new("B", FlatSamples::F32(b_vec));

    let channels = AnyChannels::sort(smallvec![r_channel, g_channel, b_channel]);

    // The layer attributes can store additional metadata
    let mut layer_attributes = LayerAttributes::named("rgb");
    layer_attributes.comments = Some("Generated by vvzen from Rust".into());
    layer_attributes.owner = Some("vvzen".into());
    layer_attributes.software_name = Some("rust-tracer".into());

    // The only layer in this image
    let layer = Layer::new(
        resolution,
        layer_attributes,
        Encoding::SMALL_LOSSLESS,
        channels,
    );

    // The layer attributes can store additional metadata
    let mut layer_attributes = LayerAttributes::named("rgb");
    layer_attributes.comments = Some("Generated by vvzen from Rust".into());
    layer_attributes.owner = Some("vvzen".into());
    layer_attributes.software_name = Some("vv-ltsr".into());

    // Write the image to disk
    let image = Image::from_layer(layer);

    Ok(image)
}
