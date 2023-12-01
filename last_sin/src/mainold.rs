use std::ffi::OsString;
use std::os::windows::ffi::{OsStrExt, OsStringExt};
use opencv::{core, highgui, imgcodecs};
use winapi::um::winuser::{FindWindowA, GetWindowRect, GetDC, ReleaseDC, GetWindowTextW, EnumWindows, FindWindowW, WNDENUMPROC};
use winapi::shared::windef::{HWND, RECT};
use std::ptr::null_mut;
use std::time::Instant;
use opencv::core::{Mat, MatTrait, MatTraitConstManual};
use winapi::ctypes::{c_int, wchar_t};
use winapi::shared::minwindef::{BOOL, LPARAM};


unsafe fn get_img() -> Mat {
    let mut hwnd: Option<HWND> = None;
    let hwnd_ptr: LPARAM = &mut hwnd as *mut Option<HWND> as LPARAM;

    unsafe {
        EnumWindows(Some(enum_windows_callback), hwnd_ptr);
    }

    let hwnd = match hwnd {
        Some(hwnd) => hwnd,
        None => return Mat::default(), // Return an empty Mat if no window is found
    };

    let mut rect = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    let (width, height, buf) = unsafe {
        GetWindowRect(hwnd, &mut rect);
        let width = rect.right - rect.left;
        let height = rect.bottom - rect.top;
        let hdc = GetDC(hwnd);
        let mut buf: Vec<u32> = vec![0; (width * height) as usize];
        winapi::um::wingdi::BitBlt(hdc, 0, 0, width, height, hdc, 0, 0, winapi::um::wingdi::SRCCOPY);
        winapi::um::wingdi::GetDIBits(hdc, null_mut(), 0, height as u32, buf.as_mut_ptr() as *mut _,
                                      &mut winapi::um::wingdi::BITMAPINFO {
                                          bmiHeader: winapi::um::wingdi::BITMAPINFOHEADER {
                                              biSize: std::mem::size_of::<winapi::um::wingdi::BITMAPINFOHEADER>() as u32,
                                              biWidth: width,
                                              biHeight: height * -1,
                                              biPlanes: 1,
                                              biBitCount: 32,
                                              biCompression: winapi::um::wingdi::BI_RGB,

                                              // ..Default::default()
                                              biSizeImage: 0,
                                              biXPelsPerMeter: 0,
                                              biYPelsPerMeter: 0,
                                              biClrUsed: 0,
                                              biClrImportant: 0,
                                          },


                                          // ..Default::default()
                                          bmiColors: [winapi::um::wingdi::RGBQUAD {
                                              rgbBlue: 0,
                                              rgbGreen: 0,
                                              rgbRed: 0,
                                              rgbReserved: 0,
                                          }; 1],
                                      }, winapi::um::wingdi::DIB_RGB_COLORS);
        ReleaseDC(hwnd, hdc);
        (width, height, buf)
    };

    let mut original_image: Mat = Mat::new_rows_cols(height as i32, width as i32, core::CV_8UC4).unwrap();

    // Copy data from `buf` to `original_image`
    for row in 0..height {
        for col in 0..width {
            let index = (row * width + col) as usize;
            let rgba = buf[index];
            let r = ((rgba >> 16) & 0xff) as u8;
            let g = ((rgba >> 8) & 0xff) as u8;
            let b = (rgba & 0xff) as u8;
            let a = ((rgba >> 24) & 0xff) as u8;
            let pixel = core::Vec4b::from([b, g, r, a]);

            let pixel_ptr = original_image.ptr_mut(row as i32).unwrap() as *mut core::Vec4b;
            *pixel_ptr = pixel;
        }
    }


    original_image
}


fn main() {
    highgui::named_window("Image", highgui::WINDOW_AUTOSIZE).unwrap();

    loop {
        let now = Instant::now();
        let img = unsafe { get_img() };

        // Get the dimensions of the image
        let size = img.size().unwrap();

        // Check if the image width and height are greater than zero
        if size.width > 0 && size.height > 0 {
            // Show the image in a window
            let cloned_img = img.clone();
            highgui::imshow("Image", &cloned_img).unwrap();
        } else {
            // println!("Failed to capture the window or the window is empty");
        }

        // print in console the time it took to process the image
        println!("{} ms", now.elapsed().as_millis());
    }
}

unsafe extern "system" fn enum_windows_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let mut title: [u16; 256] = [0; 256];
    GetWindowTextW(hwnd, title.as_mut_ptr(), title.len() as i32);

    let target_title = std::ffi::OsStr::new("My Password - KeePass")
        .encode_wide()
        .chain(std::iter::once(0))
        .collect::<Vec<u16>>();

    if title.starts_with(&target_title) {
        let hwnd_option_ptr = lparam as *mut Option<HWND>;
        *hwnd_option_ptr = Some(hwnd);
        0
    } else {
        1
    }
}