pub const FONT_BYTES: &[u8; 283684] = include_bytes!("../media/FiraCode-Medium.ttf");

pub const RENDER_BUFFER_WIDTH: usize = 2048;
pub const RENDER_BUFFER_HEIGHT: usize = 2048;
pub const RENDER_BUFFER_SIZE: usize = RENDER_BUFFER_WIDTH * RENDER_BUFFER_HEIGHT * 4;

pub const NUM_SAMPLES_PER_PIXEL: usize = 32;
