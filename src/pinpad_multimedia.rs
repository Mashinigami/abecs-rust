// src/pinpad_multimedia.rs
use crate::commands::CheckEventExtendedEvent;
use crate::protocol::calculate_crc16;
use crate::qrcode::generate_custom_qrcode;
use crate::{AbecsCommand, MultimediaFileType, PinpadConnection};
use image::ImageFormat;
use std::io::Cursor;

const MULTIMEDIA_BLOCK_SIZE: usize = 989;
const QRCODE_FILE_NAME: &str = "QRCODE01";
const PNG_FILE_NAME: &str = "IMAGE001";

pub(crate) fn write_qrcode_impl(
    port_name: &str,
    message: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let file_data_raw = generate_custom_qrcode(message)?;

    let mut file_data = Vec::new();
    file_data_raw.write_to(&mut Cursor::new(&mut file_data), ImageFormat::Png)?;

    display_png_file(port_name, QRCODE_FILE_NAME, &file_data, false)?;

    Ok(())
}

pub(crate) fn write_png_impl(
    port_name: &str,
    file_data: &[u8],
) -> Result<(), Box<dyn std::error::Error>> {
    display_png_file(port_name, PNG_FILE_NAME, file_data, false)?;

    Ok(())
}

pub(crate) fn write_png_with_keypress_impl(
    port_name: &str,
    file_data: &[u8],
) -> Result<i32, Box<dyn std::error::Error>> {
    let mut pinpad = PinpadConnection::open(port_name)?;

    open_pinpad(&mut pinpad)?;
    load_and_display_png(&mut pinpad, PNG_FILE_NAME, file_data)?;
    delete_multimedia_file(&mut pinpad, PNG_FILE_NAME)?;

    let cmd = AbecsCommand::CheckEventExtended::key().with_timeout(60);
    let key_result = pinpad.execute_typed(&cmd);

    let cmd = AbecsCommand::ClearDisplay::new();
    let clear_result = pinpad.execute_typed(&cmd);

    let cmd = AbecsCommand::Close::new();
    let close_result = pinpad.execute_typed(&cmd);

    let key_response = match key_result {
        Ok(key_response) => key_response,
        Err(err) => {
            let _ = clear_result;
            let _ = close_result;
            return Err(Box::new(err));
        }
    };

    clear_result?;
    close_result?;

    let response = match key_response.event {
        CheckEventExtendedEvent::OkEnter => 1,
        CheckEventExtendedEvent::Cancel => 0,
        _ => -1,
    };

    Ok(response)
}

fn display_png_file(
    port_name: &str,
    file_name: &str,
    file_data: &[u8],
    clear_display_before_close: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut pinpad = PinpadConnection::open(port_name)?;

    open_pinpad(&mut pinpad)?;
    load_and_display_png(&mut pinpad, file_name, file_data)?;
    delete_multimedia_file(&mut pinpad, file_name)?;

    if clear_display_before_close {
        let cmd = AbecsCommand::ClearDisplay::new();
        pinpad.execute_typed(&cmd)?;
    }

    close_pinpad(&mut pinpad)?;

    Ok(())
}

fn open_pinpad(pinpad: &mut PinpadConnection) -> Result<(), Box<dyn std::error::Error>> {
    let cmd = AbecsCommand::Open::new();
    pinpad.execute_typed(&cmd)?;

    Ok(())
}

fn close_pinpad(pinpad: &mut PinpadConnection) -> Result<(), Box<dyn std::error::Error>> {
    let cmd = AbecsCommand::Close::new();
    pinpad.execute_typed(&cmd)?;

    Ok(())
}

fn load_and_display_png(
    pinpad: &mut PinpadConnection,
    file_name: &str,
    file_data: &[u8],
) -> Result<(), Box<dyn std::error::Error>> {
    load_multimedia_png(pinpad, file_name, file_data)?;

    let cmd = AbecsCommand::DisplayImage::new(file_name);
    pinpad.execute_typed(&cmd)?;

    Ok(())
}

fn load_multimedia_png(
    pinpad: &mut PinpadConnection,
    file_name: &str,
    file_data: &[u8],
) -> Result<(), Box<dyn std::error::Error>> {
    let file_size = file_data.len() as u32;
    let file_crc = calculate_crc16(file_data);

    let cmd = AbecsCommand::MultimediaLoadInit::new(
        file_name,
        file_size,
        file_crc,
        MultimediaFileType::Png,
    );
    pinpad.execute_typed(&cmd)?;

    for chunk in file_data.chunks(MULTIMEDIA_BLOCK_SIZE) {
        let cmd = AbecsCommand::MultimediaLoadRecord::from_single(chunk.to_vec());
        pinpad.execute_typed(&cmd)?;
    }

    let cmd = AbecsCommand::MultimediaLoadEnd::new();
    pinpad.execute_typed(&cmd)?;

    Ok(())
}

fn delete_multimedia_file(
    pinpad: &mut PinpadConnection,
    file_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let cmd = AbecsCommand::DeleteMultimediaFiles::single(file_name);
    pinpad.execute_typed(&cmd)?;

    Ok(())
}