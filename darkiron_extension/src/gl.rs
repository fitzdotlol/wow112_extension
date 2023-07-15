#![allow(dead_code)]

use std::ffi::{CString, c_void};

use once_cell::sync::Lazy;
use windows::core::PCSTR;
use windows::Win32::Graphics::OpenGL::wglGetProcAddress;

pub const UNSIGNED_INT_8_8_8_8_REV: u32 = 0x8367;
pub const UNSIGNED_SHORT_4_4_4_4_REV: u32 = 0x8365;
pub const UNSIGNED_SHORT_1_5_5_5_REV: u32 = 0x8366;
pub const DSDT_NV: u32 = 0x86F5;
pub const BGRA: u32 = 0x80E1;
pub const UNSIGNED_SHORT_5_6_5: u32 = 0x8363;
pub const COMPRESSED_RGBA_S3TC_DXT1_EXT: u32 = 0x83F1;
pub const COMPRESSED_RGBA_S3TC_DXT3_EXT: u32 = 0x83F2;
pub const COMPRESSED_RGBA_S3TC_DXT5_EXT: u32 = 0x83F3;
pub const DSDT8_NV: u32 = 0x8709;
pub const TEXTURE_CUBE_MAP: u32 = 0x8513;
pub const TEXTURE_RECTANGLE: u32 = 0x84F5;
pub const ARRAY_BUFFER: u32 = 0x8892;
pub const STATIC_DRAW: u32 = 0x88E4;

///

fn get_proc_address(name: &str) -> unsafe extern "system" fn() -> isize {
    let name2 = CString::new(name).unwrap();
    let name3 = PCSTR(name2.as_ptr() as *const u8);
    let addr = unsafe { wglGetProcAddress(name3).unwrap() };

    return addr;
}

///

type def_glGenBuffers = unsafe extern "system" fn(n: u32, buffers: *mut u32);
static mut glGenBuffers: Lazy<def_glGenBuffers> = Lazy::new(|| {
    let addr = get_proc_address("glGenBuffers");
    return unsafe { std::mem::transmute(addr) };
});

type def_glBindBuffer = unsafe extern "system" fn(target: u32, buffer: u32);
static mut glBindBuffer: Lazy<def_glBindBuffer> = Lazy::new(|| {
    let addr = get_proc_address("glBindBuffer");
    return unsafe { std::mem::transmute(addr) };
});

type def_glBufferData = unsafe extern "system" fn(target: u32, size: u32, data: *const c_void, usage: u32);
static mut glBufferData: Lazy<def_glBufferData> = Lazy::new(|| {
    let addr = get_proc_address("glBufferData");
    return unsafe { std::mem::transmute(addr) };
});

type def_glDrawArrays = unsafe extern "system" fn(mode: u32, first: u32, count: u32);
static mut glDrawArrays: Lazy<def_glDrawArrays> = Lazy::new(|| {
    let addr = get_proc_address("glDrawArrays");
    return unsafe { std::mem::transmute(addr) };
});
///

pub fn gen_buffers(n: u32) -> Vec<u32> {
    let mut buffers = Vec::new();
    buffers.resize(n as usize, 0);

    unsafe {
        glGenBuffers(n, buffers.as_mut_ptr());
    }

    buffers
}

pub fn gen_buffer() -> u32 {
    let mut buffer = 0u32;

    unsafe {
        glGenBuffers(1, &mut buffer);
    }

    buffer
}

pub fn bind_buffer(target: u32, buffer: u32) {
    unsafe {
        glBindBuffer(target, buffer);
    }
}

pub fn buffer_data<T>(target: u32, data: &Vec<T>, usage: u32) {
    unsafe {
        glBufferData(
            target,
            (data.len() * std::mem::size_of::<T>()) as u32,
            data.as_ptr() as *const c_void,
            usage
        );
    }
}

// gl::draw_arrays(GL_QUADS, 0, 4);

pub fn draw_arrays(mode: u32, first: u32, count: u32) {
    unsafe {
        glDrawArrays(mode, first, count);
    }
}