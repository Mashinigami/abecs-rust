// src/pinpad_jni.rs
use crate::pinpad_display::{
    clean_display_impl,
    write_message_free_impl,
    write_message_impl,
};
use crate::pinpad_multimedia::{
    write_png_impl,
    write_png_with_keypress_impl,
};
use crate::PinpadConnection;
use jni::objects::{JByteArray, JClass, JString};
use jni::sys::{jboolean, jint};
use jni::JNIEnv;

pub(crate) extern "system" fn write_message(
    mut env: JNIEnv,
    _class: JClass,
    message: JString,
) -> jboolean {
    let Some(port_storage) = PinpadConnection::autodetect() else {
        eprintln!("Erro: Nenhum Pinpad foi encontrado automaticamente.");
        return 0;
    };

    let message_str: String = match env.get_string(&message) {
        Ok(jni_str) => jni_str.into(),
        Err(_) => return 0,
    };

    match write_message_impl(port_storage.as_str(), &message_str) {
        Ok(_) => 1,
        Err(error) => {
            eprintln!("Erro no Pinpad: {:?}", error);
            0
        }
    }
}

pub(crate) extern "system" fn write_message_free(
    mut env: JNIEnv,
    _class: JClass,
    message: JString,
) -> jboolean {
    let Some(port_storage) = PinpadConnection::autodetect() else {
        eprintln!("Erro: Nenhum Pinpad foi encontrado automaticamente.");
        return 0;
    };

    let message_str: String = match env.get_string(&message) {
        Ok(jni_str) => jni_str.into(),
        Err(_) => return 0,
    };

    match write_message_free_impl(port_storage.as_str(), &message_str) {
        Ok(_) => 1,
        Err(error) => {
            eprintln!("Erro no Pinpad: {:?}", error);
            0
        }
    }
}

pub(crate) extern "system" fn clean_display(
    _env: JNIEnv,
    _class: JClass,
) -> jboolean {
    let Some(port_storage) = PinpadConnection::autodetect() else {
        eprintln!("Erro: Nenhum Pinpad foi encontrado automaticamente.");
        return 0;
    };

    match clean_display_impl(port_storage.as_str()) {
        Ok(_) => 1,
        Err(error) => {
            eprintln!("Erro ao limpar tela: {:?}", error);
            0
        }
    }
}

pub(crate) extern "system" fn write_png(
    env: JNIEnv,
    _class: JClass,
    png_bytes: JByteArray,
) -> jboolean {
    let Some(port_storage) = PinpadConnection::autodetect() else {
        eprintln!("Erro: Nenhum Pinpad foi encontrado automaticamente.");
        return 0;
    };

    let file_data = match env.convert_byte_array(png_bytes) {
        Ok(bytes) => bytes,
        Err(error) => {
            eprintln!("Erro ao ler array de bytes da imagem: {:?}", error);
            return 0;
        }
    };

    match write_png_impl(port_storage.as_str(), &file_data) {
        Ok(_) => 1,
        Err(error) => {
            eprintln!("Erro ao mostrar imagem: {:?}", error);
            0
        }
    }
}

pub(crate) extern "system" fn write_png_with_keypress(
    env: JNIEnv,
    _class: JClass,
    png_bytes: JByteArray,
) -> jint {
    let Some(port_storage) = PinpadConnection::autodetect() else {
        eprintln!("Erro: Nenhum Pinpad foi encontrado automaticamente.");
        return -1;
    };

    let file_data = match env.convert_byte_array(png_bytes) {
        Ok(bytes) => bytes,
        Err(error) => {
            eprintln!("Erro ao ler array de bytes da imagem: {:?}", error);
            return -1;
        }
    };

    write_png_with_keypress_impl(port_storage.as_str(), &file_data).unwrap_or_else(|error| {
        eprintln!("Erro ao mostrar imagem: {:?}", error);
        -1
    })
}