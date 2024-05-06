use std::time::Duration;

use find_subimage::SubImageFinderState;
use image::io::Reader as ImageReader;
use image::{DynamicImage, RgbImage, RgbaImage};
use rand::{self, thread_rng, Rng};
use win_screenshot::prelude::*;
use windows::core::PCWSTR;
use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    FindWindowExW, PostMessageA, SendMessageA, SetWindowPos, SWP_NOMOVE, SWP_NOZORDER,
    WA_CLICKACTIVE, WM_ACTIVATE, WM_LBUTTONDOWN, WM_LBUTTONUP,
};

fn main() {
    let main_hwnd = find_window("쿠키런").expect("게임을 찾지 못했습니다");
    let game_hwnd;
    unsafe {
        let window_name = "HD-Player"
            .encode_utf16()
            .chain(Some(0))
            .collect::<Vec<_>>();
        game_hwnd = FindWindowExW(
            HWND(main_hwnd),
            HWND::default(),
            PCWSTR::null(),
            PCWSTR(window_name.as_ptr()),
        )
        .0;
    }

    if game_hwnd == 0 {
        panic!("게임을 찾지 못했습니다");
    }

    println!("hwnd: {:#X}", game_hwnd);

    unsafe {
        SetWindowPos(
            HWND(main_hwnd),
            HWND(0),
            0,
            0,
            1280,
            720,
            SWP_NOMOVE | SWP_NOZORDER,
        )
        .ok();
    }

    let images = get_images("images").expect("이미지 가져오기 실패");
    println!("이미지 개수: {}", images.len());

    let mut finder = SubImageFinderState::new_opencv(None);
    let to_tuple: fn(&image::ImageBuffer<_, _>) -> (&Vec<u8>, usize, usize) =
        |img| (img.as_raw(), img.width() as usize, img.height() as usize);

    loop {
        std::thread::sleep(Duration::from_secs(3));
        let buf = capture_window(game_hwnd);
        if let Ok(buf) = buf {
            if let Some(img) = RgbaImage::from_raw(buf.width, buf.height, buf.pixels) {
                let screen = DynamicImage::ImageRgba8(img).to_rgb8();

                'outer: for image in images.iter() {
                    let positions =
                        finder.find_subimage_positions(to_tuple(&screen), to_tuple(image), 3);

                    for position in positions.iter() {
                        let x_min = position.0;
                        let x_max = position.0 + image.width() as usize;

                        let y_min = position.1;
                        let y_max = position.1 + image.height() as usize;
                        let pos = (
                            thread_rng().gen_range(x_min..=x_max),
                            thread_rng().gen_range(y_min..=y_max),
                        );
                        println!("click {} {}", pos.0, pos.1);
                        click(game_hwnd, pos.0 as isize, pos.1 as isize);
                        break 'outer;
                    }
                }
            }
        }
    }
}

fn click(hwnd: isize, x: isize, y: isize) {
    let pos = x | (y << 16);

    unsafe {
        SendMessageA(
            HWND(hwnd),
            WM_ACTIVATE,
            WPARAM(WA_CLICKACTIVE as usize),
            LPARAM(0),
        );
        std::thread::sleep(Duration::from_millis(thread_rng().gen_range(100..=300)));
        PostMessageA(HWND(hwnd), WM_LBUTTONDOWN, WPARAM(1), LPARAM(pos)).ok();
        std::thread::sleep(Duration::from_millis(thread_rng().gen_range(70..=200)));
        PostMessageA(HWND(hwnd), WM_LBUTTONUP, WPARAM(0), LPARAM(pos)).ok();
    }
}

fn get_images(path: &str) -> Result<Vec<RgbImage>, std::io::Error> {
    let mut images: Vec<_> = vec![];
    for entry in std::fs::read_dir(path)? {
        let dir = entry?;
        if let Some(filename) = dir.file_name().to_str() {
            if filename.ends_with(".png") {
                if let Ok(img) = ImageReader::open(dir.path()) {
                    if let Ok(img) = img.decode() {
                        images.push(img.to_rgb8());
                    }
                }
            }
        }
    }

    Ok(images)
}
