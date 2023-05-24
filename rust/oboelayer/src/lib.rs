mod wav_player;

extern crate jni;
#[macro_use]
extern crate log;

use std::ffi::{CStr, CString};
use std::os::raw::c_char;

use jni::objects::{JClass, JObject, JString, JValue, JValueGen};
use jni::{JNIEnv};
use jni::sys::jstring;
use crate::wav_player::WavPlayer;

static mut WAV: Option<WavPlayer> = Some(WavPlayer::new());

#[no_mangle]
#[allow(non_snake_case)]
pub extern "C" fn Java_ru_teplicate_oboemanipulator_NativeLayer_playWav(mut env: JNIEnv, _class: JClass, path: JString) {
    let c_str = env.get_string(&path).unwrap().as_ptr();
    let output = unsafe { CStr::from_ptr(c_str).to_str().unwrap() };

    unsafe {
        if let Some(w) = WAV.as_mut() {
            w.play(output.to_string());
        }
    }
}


