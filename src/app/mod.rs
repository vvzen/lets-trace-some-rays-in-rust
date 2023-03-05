use std::path::PathBuf;

use iced::futures;
use iced::theme::Theme;
use iced::widget::{button, column, container, image, progress_bar, row, text, text_input};
use iced::{Application, Command, Element, Length};

use crate::app::filesystem::save_exr_image_to_disk;
use crate::app::rendering::{convert_to_openexr, RenderTask};
use crate::constants::{RENDER_BUFFER_HEIGHT, RENDER_BUFFER_SIZE, RENDER_BUFFER_WIDTH};

mod filesystem;
mod rendering;

#[derive(Debug, Clone)]
pub enum AppError {
    RenderError,
}

#[derive(Debug, Clone)]
pub enum Message {
    FileNameChanged(String),
    SaveFilePressed,
    RenderPressed,
    RenderTaskFinished(Result<Vec<f32>, AppError>),
    DisplayConversionTaskFinished(Result<Vec<u8>, AppError>),
}

/// Stores the state of the Application (GUI and all)
pub struct LTSRApp {
    pub file_name: String,
    pub file_name_with_ext: String,
    pub current_render_progress: f32,
    pub render_progress_label: String,

    /// 8bit image displayed in the GUI
    pub rendered_image: image::Handle,
    /// 32bit floating point render buffer storing the rendered image
    pub render_buffer: Vec<f32>,
}

impl Application for LTSRApp {
    type Message = Message;
    type Executor = iced::executor::Default;
    type Flags = ();
    type Theme = Theme;

    fn new(_flags: ()) -> (LTSRApp, Command<Message>) {
        let file_name = String::from("sample_file");

        let render_buffer: Vec<f32> = vec![0.0; RENDER_BUFFER_SIZE];
        let display_buffer: Vec<u8> = vec![0; RENDER_BUFFER_SIZE];

        // Creates an image Handle containing the image pixels directly.
        // This function expects the input data to be provided as a Vec<u8> of RGBA pixels.
        let image = image::Handle::from_pixels(
            RENDER_BUFFER_WIDTH as u32,
            RENDER_BUFFER_HEIGHT as u32,
            display_buffer.clone(),
        );

        let render_progress_label = String::from("Render not started.");

        (
            LTSRApp {
                file_name: file_name.clone(),
                file_name_with_ext: format!("{file_name}.exr"),
                current_render_progress: 0.0,
                rendered_image: image,
                render_buffer,
                render_progress_label,
            },
            Command::none(),
        )
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

        // Progress Report
        // let render_progress_bar = progress_bar(0.0..=100.0, self.current_render_progress);
        let render_progress_label = container(text(&self.render_progress_label).size(12))
            .width(Length::Fill)
            .center_x();

        // Save text field
        let file_name_input = text_input(
            "Your file name",
            &self.file_name,
            Self::Message::FileNameChanged,
        )
        .padding(10)
        .size(20);

        // Save button
        let save_button = button(
            text("Save")
                .width(Length::Fill)
                .horizontal_alignment(iced::alignment::Horizontal::Center),
        )
        .on_press(Self::Message::SaveFilePressed)
        .padding(10)
        .width(100);

        // Final UI
        let content = column![
            row![rendered_image].padding(10).spacing(10),
            row![render_progress_label].padding(10).spacing(10),
            // row![render_progress_bar].padding(10).spacing(10),
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

    fn update(&mut self, message: Message) -> Command<Self::Message> {
        match message {
            Message::RenderTaskFinished(Ok(render_buffer)) => {
                self.render_progress_label =
                    String::from("Main ACEScg Render finished, converting to sRGB..");

                Command::perform(
                    RenderTask::convert_to_display_buffer(render_buffer),
                    Message::DisplayConversionTaskFinished,
                )
            }
            Message::RenderTaskFinished(Err(err)) => {
                eprintln!("Render failed: {err:?}");
                Command::none()
            }
            Message::DisplayConversionTaskFinished(Ok(display_buffer)) => {
                self.rendered_image = image::Handle::from_pixels(
                    RENDER_BUFFER_WIDTH as u32,
                    RENDER_BUFFER_HEIGHT as u32,
                    display_buffer.clone(),
                );

                self.render_progress_label =
                    String::from("Converted from ACEScg to Display Color Space!");

                Command::none()
            }

            Message::DisplayConversionTaskFinished(Err(err)) => {
                // TODO:
                Command::none()
            }

            Message::RenderPressed => {
                let message = String::from("Starting new Render in the background..");
                self.render_progress_label = message;

                // Schedule the background render
                Command::perform(RenderTask::render_scene(), Message::RenderTaskFinished)
            }
            Message::FileNameChanged(new_name) => {
                self.file_name = new_name;
                self.file_name_with_ext = format!("{}.exr", self.file_name);

                Command::none()
            }
            Message::SaveFilePressed => {
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

                Command::none()
            }
        }
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }
}
