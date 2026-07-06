use qrcode::{QrCode, Color};
use qrcode::types::QrError;
use image::{GrayImage, Luma};
use image::imageops::FilterType;

/// Generates a QR Code image resized to exactly 192x272 pixels.
pub fn generate_custom_qrcode(text: &str) -> Result<GrayImage, QrError> {
    // 1. Generate the QR Code matrix from the input text
    let code = QrCode::new(text.as_bytes())?;

    // 2. Convert the matrix into a black and white image buffer
    let qr_size = code.width() as u32;
    let mut base_image = GrayImage::new(qr_size, qr_size);

    for y in 0..qr_size {
        for x in 0..qr_size {
            let pixel = match code[(x as usize, y as usize)] {
                Color::Dark => Luma([0]),    // Black
                Color::Light => Luma([255]), // White
            };
            base_image.put_pixel(x, y, pixel);
        }
    }

    // 3. Resize the canvas to 192x192 pixels
    let resized_qr = image::imageops::resize(
        &base_image,
        192,
        192,
        FilterType::Nearest
    );

    Ok(resized_qr)
}