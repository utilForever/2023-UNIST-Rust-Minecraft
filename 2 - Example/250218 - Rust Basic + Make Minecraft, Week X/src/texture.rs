use crate::gl_call;

use gl;
use image::ColorType;
use std::os::raw::c_void;

pub fn create_texture(path: &str) -> u32 {
    let mut id = 0;

    gl_call!(gl::CreateTextures(gl::TEXTURE_2D, 1, &mut id));
    gl_call!(gl::TextureParameteri(
        id,
        gl::TEXTURE_MIN_FILTER,
        gl::NEAREST_MIPMAP_NEAREST as i32
    ));
    gl_call!(gl::TextureParameteri(
        id,
        gl::TEXTURE_MAG_FILTER,
        gl::NEAREST as i32
    ));

    let img = image::open(path);
    let img = match img {
        Ok(img) => img.flipv(),
        Err(err) => panic!("Filename: {path}, error: {err}"),
    };

    match img.color() {
        ColorType::Rgba8 => {}
        _ => panic!("Texture format not supported"),
    };

    gl_call!(gl::TextureStorage2D(
        id,
        1,
        gl::RGBA8,
        img.width() as i32,
        img.height() as i32
    ));
    gl_call!(gl::TextureSubImage2D(
        id,
        0,
        0,
        0,
        img.width() as i32,
        img.height() as i32,
        gl::RGBA,
        gl::UNSIGNED_BYTE,
        img.as_bytes().as_ptr() as *mut c_void
    ));
    gl_call!(gl::GenerateTextureMipmap(id));

    id
}
