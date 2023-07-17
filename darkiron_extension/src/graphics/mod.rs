#![allow(dead_code)]
use std::ffi::{c_char, c_void, CString};

use once_cell::sync::Lazy;
use windows::core::PCSTR;
use windows::Win32::Foundation::{BOOL, HWND, LPARAM, WPARAM};
use windows::Win32::Graphics::Gdi::{CreateBitmap, CreateCompatibleBitmap, GetDC};
use windows::Win32::System::LibraryLoader::GetModuleHandleA;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateIconIndirect, CreateWindowExA, SendMessageA, CW_USEDEFAULT, HMENU, ICONINFO, ICON_BIG,
    ICON_SMALL, WM_SETICON, WS_CAPTION, WS_EX_APPWINDOW, WS_EX_OVERLAPPEDWINDOW, WS_MAXIMIZEBOX,
    WS_MINIMIZEBOX, WS_OVERLAPPED, WS_SYSMENU, WS_THICKFRAME,
};

use darkiron_macro::detour_fn;

use crate::config::CONFIG;
use crate::math::{Matrix4, RectI};
use crate::ui::UIState;

pub mod gl;
pub mod gx;
pub mod texture;
pub mod primitive;

pub fn create_orthographic_projection(near: f32, far: f32) -> Matrix4 {
    let mut projection = Matrix4 { m: [[0.0; 4]; 4] };

    // window rect
    let r = unsafe { &*std::mem::transmute::<u32, *const RectI>(0x00884E20) };

    let left = r.x1 as f32;
    let right = r.x2 as f32;
    let top = r.y1 as f32;
    let bottom = r.y2 as f32;

    projection.m[0][0] = 2.0 / (right - left);
    projection.m[1][1] = 2.0 / (top - bottom);
    projection.m[2][2] = 1.0 / (far - near);
    projection.m[3][0] = -(right + left) / (right - left);
    projection.m[3][1] = -(top + bottom) / (top - bottom);
    projection.m[3][2] = -near / (far - near);
    projection.m[3][3] = 1.0;

    projection
}

static mut UI: Lazy<UIState> = Lazy::new(|| {
    UIState::new()
});

fn set_window_icon(hwnd: HWND) {
    if CONFIG.icon.is_none() {
        return;
    }

    let icon_path = CONFIG.icon.as_ref().unwrap();

    let img = image::io::Reader::open(icon_path)
        .unwrap()
        .decode()
        .unwrap();
    let pixels = img.clone().into_rgba8().as_ptr() as *const c_void;

    let hbmColor =
        unsafe { CreateBitmap(img.width() as i32, img.height() as i32, 1, 32, Some(pixels)) };
    let hbmMask = unsafe { CreateCompatibleBitmap(UI.dc, img.width() as i32, img.height() as i32) };

    let icon_info = ICONINFO {
        fIcon: BOOL::from(true),
        xHotspot: 0,
        yHotspot: 0,
        hbmMask,
        hbmColor,
    };

    let icon = unsafe { CreateIconIndirect(&icon_info).unwrap() };
    let lp = LPARAM(icon.0);

    unsafe {
        SendMessageA(hwnd, WM_SETICON, WPARAM(ICON_BIG as usize), lp);
        SendMessageA(hwnd, WM_SETICON, WPARAM(ICON_SMALL as usize), lp);
    }
}

#[detour_fn(0x0058CF10)]
unsafe extern "fastcall" fn z_recreateOpenglWindow(
    this: *const c_void,
    win: *const gx::OpenGlWindow,
) -> HWND {
    // EnumWindows
    let hinstance = GetModuleHandleA(PCSTR(std::ptr::null())).unwrap();
    let class_name = "GxWindowClassOpenGl\0";

    // console_write("[ui] windows:", crate::ConsoleColor::Warning);
    // let text = format!("  * {}", err_str.to_str().unwrap());
    // console_write(&text, crate::console::ConsoleColor::Error);

    let hwnd = CreateWindowExA(
        WS_EX_APPWINDOW | WS_EX_OVERLAPPEDWINDOW,
        PCSTR(class_name.as_ptr()),
        PCSTR(class_name.as_ptr()),
        WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_THICKFRAME | WS_MINIMIZEBOX | WS_MAXIMIZEBOX,
        CW_USEDEFAULT,
        CW_USEDEFAULT,
        (*win).width,
        (*win).height,
        HWND(0),
        HMENU(0),
        hinstance,
        Some(this),
    );

    UI.window = hwnd;
    UI.dc = GetDC(hwnd);

    set_window_icon(hwnd);

    return hwnd;
}

#[detour_fn(0x0059BA10)]
unsafe extern "thiscall" fn sub_59BA10(dev_ptr: u32, a2: u32) -> u32 {
    // _ = UI.draw();

    hook_sub_59BA10.disable().unwrap();
    let ret_val = hook_sub_59BA10.call(dev_ptr, a2);
    hook_sub_59BA10.enable().unwrap();

    ret_val
}

//int __fastcall sub_435A50(int a1, char *windowTitle)
#[detour_fn(0x00435A50)]
extern "fastcall" fn sub_435A50(a1: u32, _windowTitle: *const c_char) -> u32 {
    let cfg = &CONFIG;

    let win_name = match &cfg.title {
        Some(s) => s.as_str(),
        None => "World of Warcraft",
    };

    let win_name = CString::new(win_name).unwrap();

    unsafe {
        hook_sub_435A50.disable().unwrap();
        let ret = hook_sub_435A50.call(a1, win_name.as_ptr());
        hook_sub_435A50.enable().unwrap();

        return ret;
    }
}

pub async fn add_rect_from_url(url: &str) -> Result<(), Box<dyn std::error::Error>> {
    unsafe { UI.add_rect_from_url(url).await?; }

    Ok(())
}

pub fn toggle_ui() {
    unsafe { UI.enabled = !UI.enabled };
}

pub fn init() {
    unsafe {
        // disable direct3d lol
        // TODO: find a less-idiotic way of doing this
        let src = "OpenGL\0";
        let dst_1 = crate::mem::ptr::<u8>(0x0080E138);
        let dst_2 = crate::mem::ptr::<u8>(0x00864F7C);
        std::ptr::copy(src.as_ptr() as *mut u8, dst_1, src.len());
        std::ptr::copy(src.as_ptr() as *mut u8, dst_2, src.len());

        hook_sub_59BA10.enable().unwrap();
        hook_z_recreateOpenglWindow.enable().unwrap();
        hook_sub_435A50.enable().unwrap();
    }
}