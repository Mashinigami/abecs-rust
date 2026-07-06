use crate::{AbecsCommand, MultimediaFileType, PinpadConnection};
use jni::objects::{JClass, JString};
use jni::sys::{jboolean, jint, JNI_VERSION_1_6};
use jni::JNIEnv;
use std::ffi::c_void;
use std::io::Cursor;
use image::ImageFormat;
use crate::protocol::calculate_crc16;
use crate::qrcode::generate_custom_qrcode;

extern "system" fn write_message(
    mut env: JNIEnv,
    _class: JClass,
    message: JString,
) -> jboolean {
    if let Some(port_storage) = PinpadConnection::autodetect() {
        let port_str: &str = port_storage.as_str();
        let message_str: String = match env.get_string(&message) {
            Ok(jni_str) => jni_str.into(),
            Err(_) => return 0,
        };

        match write_message_impl(&port_str, &message_str) {
            Ok(_) => 1,
            Err(error) => {
                eprintln!("Erro no Pinpad: {:?}", error);
                0
            }
        }
    } else {
        eprintln!("Erro: Nenhum Pinpad foi encontrado automaticamente.");
        0
    }
}

extern "system" fn write_message_free(
    mut env: JNIEnv,
    _class: JClass,
    message: JString,
) -> jboolean {
    if let Some(port_storage) = PinpadConnection::autodetect() {
        let port_str: &str = port_storage.as_str();
        let message_str: String = match env.get_string(&message) {
            Ok(jni_str) => jni_str.into(),
            Err(_) => return 0,
        };

        match write_message_free_impl(&port_str, &message_str) {
            Ok(_) => 1,
            Err(error) => {
                eprintln!("Erro no Pinpad: {:?}", error);
                0
            }
        }
    } else {
        eprintln!("Erro: Nenhum Pinpad foi encontrado automaticamente.");
        0
    }
}

extern "system" fn clean_display(
    mut _env: JNIEnv,
    _class: JClass,
) -> jboolean {
    if let Some(port_storage) = PinpadConnection::autodetect() {
        let port_str: &str = port_storage.as_str();

        match clean_display_impl(port_str) {
            Ok(_) => 1,
            Err(error) => {
                eprintln!("Erro ao limpar tela: {:?}", error);
                0
            }
        }
    } else {
        eprintln!("Erro: Nenhum Pinpad foi encontrado automaticamente.");
        0
    }
}

extern "system" fn write_qrcode(
    mut env: JNIEnv,
    _class: JClass,
    message: JString,
) -> jboolean {
    if let Some(port_storage) = PinpadConnection::autodetect() {
        let port_str: &str = port_storage.as_str();
        let message_str: String = match env.get_string(&message) {
            Ok(jni_str) => jni_str.into(),
            Err(_) => return 0,
        };

        match write_qrcode_impl(port_str, &message_str) {
            Ok(_) => 1,
            Err(error) => {
                eprintln!("Erro ao mostrar QR Code: {:?}", error);
                0
            }
        }
    } else {
        eprintln!("Erro: Nenhum Pinpad foi encontrado automaticamente.");
        0
    }
}

fn write_message_impl(port_name: &str, message: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut pinpad = PinpadConnection::open(port_name)?;
    let cmd = AbecsCommand::Open::new();
    let _response = pinpad.execute_typed(&cmd)?;
    let cmd = AbecsCommand::Display::new(message);
    pinpad.execute_typed(&cmd)?;
    let cmd = AbecsCommand::Close::new();
    pinpad.execute_typed(&cmd)?;

    Ok(())
}

fn write_message_free_impl(port_name: &str, message: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut pinpad = PinpadConnection::open(port_name)?;
    let cmd = AbecsCommand::Open::new();
    let _response = pinpad.execute_typed(&cmd)?;
    let cmd = AbecsCommand::DisplayMessage::new(message);
    pinpad.execute_typed(&cmd)?;
    let cmd = AbecsCommand::Close::new();
    pinpad.execute_typed(&cmd)?;

    Ok(())
}

fn clean_display_impl(port_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut pinpad = PinpadConnection::open(port_name)?;
    let cmd = AbecsCommand::Open::new();
    pinpad.execute_typed(&cmd)?;
    let cmd = AbecsCommand::ClearDisplay::new();
    pinpad.execute_typed(&cmd)?;
    let cmd = AbecsCommand::Close::new();
    pinpad.execute_typed(&cmd)?;

    Ok(())
}

fn write_qrcode_impl(port_name: &str, message: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut pinpad = PinpadConnection::open(port_name)?;
    let cmd = AbecsCommand::Open::new();
    pinpad.execute_typed(&cmd)?;
    let file_data_raw = generate_custom_qrcode(&message)?;
    let mut file_data = Vec::new();
    file_data_raw.write_to(&mut Cursor::new(&mut file_data), ImageFormat::Png)
        .expect("Failed to encode PNG");
    let file_size = file_data.len() as u32;
    let file_crc = calculate_crc16(&file_data);
    let file_name = "QRCODE01";
    let cmd = AbecsCommand::MultimediaLoadInit::new(
        file_name,
        file_size,
        file_crc,
        MultimediaFileType::Png,
    );
    pinpad.execute_typed(&cmd)?;
    let block_size = 989;
    for (_i, chunk) in file_data.chunks(block_size).enumerate() {
        let cmd = AbecsCommand::MultimediaLoadRecord::from_single(chunk.to_vec());
        pinpad.execute_typed(&cmd)?;
    }
    let cmd = AbecsCommand::MultimediaLoadEnd::new();
    pinpad.execute_typed(&cmd)?;
    let cmd = AbecsCommand::DisplayImage::new(file_name);
    pinpad.execute_typed(&cmd)?;
    let cmd = AbecsCommand::DeleteMultimediaFiles::single(file_name);
    pinpad.execute_typed(&cmd)?;
    let cmd = AbecsCommand::Close::new();
    pinpad.execute_typed(&cmd)?;

    Ok(())
}

#[no_mangle]
pub extern "system" fn JNI_OnLoad(vm: jni::JavaVM, _reserved: *mut c_void) -> jint {
    if let Ok(mut env) = vm.get_env() {
        let target_class = "br/com/fsj/estacao/server/tef/PinpadService";

        if let Ok(clazz) = env.find_class(target_class) {
            let methods = [
                jni::NativeMethod {
                    name: jni::strings::JNIString::from("writeMessage"),
                    sig: jni::strings::JNIString::from("(Ljava/lang/String;)Z"),
                    fn_ptr: write_message as *mut c_void,
                },
                jni::NativeMethod {
                    name: jni::strings::JNIString::from("writeMessageFree"),
                    sig: jni::strings::JNIString::from("(Ljava/lang/String;)Z"),
                    fn_ptr: write_message_free as *mut c_void,
                },
                jni::NativeMethod {
                    name: jni::strings::JNIString::from("cleanDisplay"),
                    sig: jni::strings::JNIString::from("()Z"),
                    fn_ptr: clean_display as *mut c_void,
                },
                jni::NativeMethod {
                    name: jni::strings::JNIString::from("writeQrCode"),
                    sig: jni::strings::JNIString::from("(Ljava/lang/String;)Z"),
                    fn_ptr: write_qrcode as *mut c_void,
                }
            ];

            if env.register_native_methods(&clazz, &methods).is_ok() {
                return JNI_VERSION_1_6;
            }
        }
    }
    -1
}