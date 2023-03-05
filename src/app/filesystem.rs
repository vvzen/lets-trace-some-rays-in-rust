use std::path::Path;

use crate::app::rendering::SimpleOpenEXRImage;
use anyhow;
use exr::prelude::WritableImage;

/// Saves a simple OpenEXR image (1 Layer, many channels) to disk
pub fn save_exr_image_to_disk(
    image: SimpleOpenEXRImage,
    image_path: impl AsRef<Path>,
) -> anyhow::Result<()> {
    let parent_dir = image_path
        .as_ref()
        .parent()
        .expect("Image path had no parent directory!");

    if !parent_dir.exists() {
        std::fs::DirBuilder::new()
            .recursive(true)
            .create(parent_dir)?;
    }

    match image.write().to_file(&image_path) {
        Ok(_) => {
            eprintln!(
                "Successfully saved image to {}",
                image_path.as_ref().display()
            );
        }
        Err(e) => {
            anyhow::bail!("Failed to write image: {e:?}");
        }
    }

    Ok(())
}
