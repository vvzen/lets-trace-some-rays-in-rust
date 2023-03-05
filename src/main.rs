use iced::Application;
use iced::Settings;

mod app;
mod constants;
mod ltsr;

use app::LTSRApp;
use constants::FONT_BYTES;

fn main() {
    let mut settings = Settings::default();
    settings.default_font = Some(FONT_BYTES);
    LTSRApp::run(settings).unwrap();
}
