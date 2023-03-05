use std::path::PathBuf;

use iced::theme::Theme;
use iced::widget::{button, column, container, image, row, text, text_input};
use iced::{Element, Length, Sandbox};

use crate::app::filesystem::save_exr_image_to_disk;
use crate::app::rendering::{convert_to_display_buffer, convert_to_openexr, render_scene};
use crate::constants::{RENDER_BUFFER_HEIGHT, RENDER_BUFFER_SIZE, RENDER_BUFFER_WIDTH};

mod filesystem;
mod rendering;

#[derive(Debug, Clone)]
pub enum ApplicationMessage {
    FileNameChanged(String),
    SaveFilePressed,
    RenderPressed,
}

/// Stores the state of the Application (GUI and all)
pub struct Application {
    pub file_name: String,
    pub file_name_with_ext: String,
    /// 8bit image displayed in the GUI
    pub rendered_image: image::Handle,
    /// 32bit floating point render buffer storing the rendered image
    pub render_buffer: Vec<f32>,
}

impl Sandbox for Application {
    type Message = ApplicationMessage;

    fn new() -> Self {
        let file_name = String::from("sample_file");

        let render_buffer: Vec<f32> = vec![1.0; RENDER_BUFFER_SIZE];
        let mut display_buffer: Vec<u8> = vec![0; RENDER_BUFFER_SIZE];
        convert_to_display_buffer(&render_buffer, &mut display_buffer);

        // Creates an image Handle containing the image pixels directly.
        // This function expects the input data to be provided as a Vec<u8> of RGBA pixels.
        let image = image::Handle::from_pixels(
            RENDER_BUFFER_WIDTH as u32,
            RENDER_BUFFER_HEIGHT as u32,
            display_buffer.clone(),
        );

        Application {
            file_name: file_name.clone(),
            file_name_with_ext: format!("{file_name}.exr"),
            rendered_image: image,
            render_buffer,
        }
    }

    fn title(&self) -> String {
        String::from("Let's Trace Some Rays in Rust")
    }

    // Description of the UI
    fn view(&self) -> Element<Self::Message> {
        // This stores the image after it has been rendered
        let image_viewer = image::Viewer::new(self.rendered_image.clone()).min_scale(1.0);

        let rendered_image = container(image_viewer)
            .width(Length::Fill)
            .center_x()
            .max_height(512)
            .max_width(800);

        // Render button
        let render_button = button(
            text("Render")
                .width(Length::Fill)
                .horizontal_alignment(iced::alignment::Horizontal::Center),
        )
        .on_press(Self::Message::RenderPressed)
        .padding(10)
        .width(Length::Fill);

        // Save text field
        let file_name_input = text_input(
            "Your file name",
            &self.file_name,
            Self::Message::FileNameChanged,
        )
        .padding(10)
        .size(20);

        let save_button = button(
            text("Save")
                .width(Length::Fill)
                .horizontal_alignment(iced::alignment::Horizontal::Center),
        )
        .on_press(Self::Message::SaveFilePressed)
        .padding(10)
        .width(100);

        let content = column![
            row![rendered_image].padding(10).spacing(10),
            row![render_button].padding(10).spacing(10),
            row![file_name_input, save_button].padding(10).spacing(10),
        ]
        .max_width(800);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }

    fn update(&mut self, message: ApplicationMessage) {
        match message {
            ApplicationMessage::RenderPressed => {
                eprintln!("Rendering in the background...");
                let render_buffer: Vec<f32> = render_scene();
                self.render_buffer = render_buffer.clone();

                let mut display_buffer: Vec<u8> = vec![0; RENDER_BUFFER_SIZE];
                convert_to_display_buffer(&render_buffer, &mut display_buffer);

                self.rendered_image = image::Handle::from_pixels(
                    RENDER_BUFFER_WIDTH as u32,
                    RENDER_BUFFER_HEIGHT as u32,
                    display_buffer.clone(),
                );
            }
            ApplicationMessage::FileNameChanged(new_name) => {
                self.file_name = new_name;
                self.file_name_with_ext = format!("{}.exr", self.file_name);
            }
            ApplicationMessage::SaveFilePressed => {
                let save_dir = PathBuf::from("outputs");
                let save_path = save_dir.join(&self.file_name_with_ext);
                eprintln!("Saving render buffer to {}", save_path.display());

                match convert_to_openexr(
                    RENDER_BUFFER_WIDTH,
                    RENDER_BUFFER_HEIGHT,
                    &self.render_buffer,
                ) {
                    Ok(image) => match save_exr_image_to_disk(image, save_path) {
                        Ok(_) => {}
                        Err(e) => {
                            eprintln!("failed to save image: {e:?}");
                        }
                    },
                    Err(e) => {
                        eprintln!("failed to save image: {e:?}");
                    }
                }
            }
        }
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }
}
