use image::imageops::FilterType;
use image::GenericImageView;
use std::io::Cursor;

pub fn resize_image_bytes_to_fit_png(
    image_bytes: &[u8],
    max_width: u32,
    max_height: u32,
    upscale: bool,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let img = image::load_from_memory(image_bytes)?;

    let (width, height) = img.dimensions();

    let scale_x = max_width as f32 / width as f32;
    let scale_y = max_height as f32 / height as f32;

    let mut scale = scale_x.min(scale_y);

    if !upscale {
        scale = scale.min(1.0);
    }

    let new_width = (width as f32 * scale).round().max(1.0) as u32;
    let new_height = (height as f32 * scale).round().max(1.0) as u32;

    let resized = img.resize_exact(
        new_width,
        new_height,
        FilterType::Lanczos3,
    );

    let mut output = Cursor::new(Vec::new());

    resized.write_to(
        &mut output,
        image::ImageFormat::Png,
    )?;

    Ok(output.into_inner())
}