// src/pinpad_display.rs
use crate::{AbecsCommand, PinpadConnection};

pub(crate) fn write_message_impl(
    port_name: &str,
    message: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut pinpad = PinpadConnection::open(port_name)?;

    let cmd = AbecsCommand::Open::new();
    pinpad.execute_typed(&cmd)?;

    let cmd = AbecsCommand::Display::new(message);
    pinpad.execute_typed(&cmd)?;

    let cmd = AbecsCommand::Close::new();
    pinpad.execute_typed(&cmd)?;

    Ok(())
}

pub(crate) fn write_message_free_impl(
    port_name: &str,
    message: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut pinpad = PinpadConnection::open(port_name)?;

    let cmd = AbecsCommand::Open::new();
    pinpad.execute_typed(&cmd)?;

    let cmd = AbecsCommand::DisplayMessage::new(message);
    pinpad.execute_typed(&cmd)?;

    let cmd = AbecsCommand::Close::new();
    pinpad.execute_typed(&cmd)?;

    Ok(())
}

pub(crate) fn clean_display_impl(port_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut pinpad = PinpadConnection::open(port_name)?;

    let cmd = AbecsCommand::Open::new();
    pinpad.execute_typed(&cmd)?;

    let cmd = AbecsCommand::ClearDisplay::new();
    pinpad.execute_typed(&cmd)?;

    let cmd = AbecsCommand::Close::new();
    pinpad.execute_typed(&cmd)?;

    Ok(())
}