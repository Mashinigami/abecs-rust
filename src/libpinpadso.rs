use crate::pinpad_jni::{
    clean_display,
    write_message,
    write_message_free,
    write_png,
    write_png_with_keypress,
    write_qrcode,
};
use jni::sys::{jint, JNI_VERSION_1_6};
use std::ffi::c_void;

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
                },
                jni::NativeMethod {
                    name: jni::strings::JNIString::from("writePng"),
                    sig: jni::strings::JNIString::from("([B)Z"),
                    fn_ptr: write_png as *mut c_void,
                },
                jni::NativeMethod {
                    name: jni::strings::JNIString::from("writePngWithKeypress"),
                    sig: jni::strings::JNIString::from("([B)I"),
                    fn_ptr: write_png_with_keypress as *mut c_void,
                },
            ];

            if env.register_native_methods(&clazz, &methods).is_ok() {
                return JNI_VERSION_1_6;
            }
        }
    }

    -1
}