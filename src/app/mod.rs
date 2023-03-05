use iced::theme::Theme;
use iced::widget::{button, column, container, image, row, text, text_input};
use iced::{Alignment, Element, Length, Sandbox};

use crate::app::rendering::{convert_to_display_buffer, render_scene};
use crate::constants::{RENDER_BUFFER_HEIGHT, RENDER_BUFFER_SIZE, RENDER_BUFFER_WIDTH};

mod rendering;

#[derive(Debug, Clone)]
pub enum ApplicationMessage {
    FileNameChanged(String),
    SaveFilePressed,
    RenderPressed,
}

pub struct ApplicationState {
    pub file_name: String,
    pub file_name_with_ext: String,
    pub rendered_image: image::Handle,
}

impl Sandbox for ApplicationState {
    type Message = ApplicationMessage;

    fn new() -> Self {
        let file_name = String::from("sample_file");

        let render_buffer: Vec<f32> = render_scene();
        let mut display_buffer: Vec<u8> = vec![0; RENDER_BUFFER_SIZE];
        convert_to_display_buffer(&render_buffer, &mut display_buffer);

        // Creates an image Handle containing the image pixels directly.
        // This function expects the input data to be provided as a Vec<u8> of RGBA pixels.
        let image = image::Handle::from_pixels(
            RENDER_BUFFER_WIDTH as u32,
            RENDER_BUFFER_HEIGHT as u32,
            display_buffer.clone(),
        );

        ApplicationState {
            file_name: file_name.clone(),
            file_name_with_ext: format!("{file_name}.exr"),
            rendered_image: image,
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
            }
            ApplicationMessage::FileNameChanged(new_name) => {
                eprintln!("New name: {new_name}");
                self.file_name = new_name;
                eprintln!("New file name: {}", self.file_name);
                self.file_name_with_ext = format!("{}.exr", self.file_name);
            }
            ApplicationMessage::SaveFilePressed => {
                eprintln!("Saving {} to disk..", self.file_name_with_ext);
            }
        }
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }
}
