pub const WINDOW_WIDTH: u32 = 1500;
pub const WINDOW_HEIGHT: u32 = 720;

pub const RENDER_BUFFER_WIDTH: u32 = 256;
pub const RENDER_BUFFER_HEIGHT: u32 = 256;
pub const RENDER_BUFFER_SIZE: usize = (RENDER_BUFFER_WIDTH * RENDER_BUFFER_HEIGHT * 4) as usize;

// 1024 * 1024 * 4         = 4_194_304
// max size of a 32bit int = 4_294_967_295
