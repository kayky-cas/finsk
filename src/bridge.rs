use raylib::{
    ffi::{GetCharPressed, LoadFontFromMemory},
    prelude::*,
};
use std::{ffi::CString, ptr::null_mut};

pub fn get_key_pressed() -> char {
    let key = unsafe { GetCharPressed() as u32 };
    char::from_u32(key).expect("all keys returned by 'GetCharPressed' should be characters")
}

pub fn load_font_from_memory(font_data: &[u8], font_size: i32, font_file_type: &str) -> Font {
    let font_ft = CString::new(font_file_type)
        .expect("the font file type shouldn't have a \\0 character in the middle of the string");
    unsafe {
        Font::from_raw(LoadFontFromMemory(
            font_ft.as_ptr(),
            font_data.as_ptr(),
            font_data
                .len()
                .try_into()
                .expect("the font data length should fit in a 32 bit integer"),
            font_size,
            null_mut(),
            100,
        ))
    }
}
