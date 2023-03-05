use iced::Sandbox;
use iced::Settings;

mod app;
mod constants;
mod ltsr;

use app::Application;
use constants::FONT_BYTES;

fn main() {
    let mut settings = Settings::default();
    settings.default_font = Some(FONT_BYTES);
    Application::run(settings).unwrap();
}
