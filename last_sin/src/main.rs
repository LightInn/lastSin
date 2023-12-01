use std::error::Error;
use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use std::ptr::{null_mut, self};
use std::slice;
use std::time::Duration;
use opencv::{core, highgui, imgcodecs, prelude::MatTrait};
use opencv::imgproc::COLOR_BGRA2BGR;
use opencv::imgproc::cvt_color;

use winapi::shared::windef::HWND;
use winapi::um::winuser::{EnumWindows, GetWindowTextW, GetWindowTextLengthW};
use windows::Graphics::Capture::*;
use windows::Graphics::DirectX::Direct3D11::{IDirect3DDevice, IDirect3DSurface};
use windows::Graphics::SizeInt32;
use winapi::shared::{dxgi1_2, dxgiformat, dxgitype, winerror};
use winapi::um::d3dcommon;
use winapi::um::d3d11::{self, ID3D11Device, ID3D11DeviceContext, ID3D11Resource};
use winapi::Interface;
use winapi::um::winnt::HRESULT;
use windows::Win32::Foundation::{BOOL, FALSE, LPARAM, TRUE};


extern "system" fn enum_windows_callback<F: FnMut(HWND) -> bool>(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let closure = unsafe { &mut *(lparam as *mut F) };
    if closure(hwnd) { TRUE } else { FALSE }
}

fn enumerate_windows<F: FnMut(HWND) -> bool>(mut f: F) {
    let lparam = &mut f as *mut _ as LPARAM;
    unsafe { EnumWindows(Some(enum_windows_callback::<F>), lparam); }
}

fn find_window(window_title: &str) -> Option<HWND> {
    let name = OsString::from(window_title);
    let mut hwnd_found: Option<HWND> = None;
    enumerate_windows(|hwnd| {
        let len = unsafe { GetWindowTextLengthW(hwnd) };
        let mut buf = vec![0u16; len as usize + 1];
        unsafe { GetWindowTextW(hwnd, buf.as_mut_ptr(), buf.len() as i32) };
        if OsString::from_wide(&buf) == name {
            hwnd_found = Some(hwnd);
            false
        } else {
            true
        }
    });
    hwnd_found
}
fn create_d3d11_device() -> Result<(*mut ID3D11Device, *mut ID3D11DeviceContext), HRESULT> {
    let mut device: *mut ID3D11Device = ptr::null_mut();
    let mut device_context: *mut ID3D11DeviceContext = ptr::null_mut();

    let result = unsafe {
        d3d11::D3D11CreateDevice(
            ptr::null_mut(),
            d3dcommon::D3D_DRIVER_TYPE_HARDWARE,
            ptr::null_mut(),
            0,
            ptr::null(),
            0,
            d3d11::D3D11_SDK_VERSION,
            &mut device,
            ptr::null_mut(),
            &mut device_context,
        )
    };

    if result < 0 {
        Err(result)
    } else {
        Ok((device, device_context))
    }
}

fn capture_window(hwnd: HWND) -> Result<core::Mat, Box<dyn Error>> {
    let capture_item = GraphicsCaptureItem::CreateFromWindowHandle(hwnd)?;

    let (d3d_device, d3d_device_context) = create_d3d11_device()
        .map_err(|e| format!("Failed to create D3D11 device: {:?}", e))?;

    let frame_pool = Direct3D11CaptureFramePool::Create(
        &d3d_device,
        capture_item.PixelFormat(),
        1,
        *capture_item.Size(),
    )?;

    let mut captured_mat: Option<core::Mat> = None;
    let frame_arrived_token = frame_pool.FrameArrived(&|frame_pool, _arg| unsafe {
        if let Ok(frame) = frame_pool.TryGetNextFrame() {
            let surface: IDirect3DSurface = frame.Surface()?;
            let surface_desc = surface.Description()?;
            let width = surface_desc.Width as i32;
            let height = surface_desc.Height as i32;

            let mut mapped_resource = d3d11::D3D11_MAPPED_SUBRESOURCE { pData: ptr::null_mut(), RowPitch: 0, DepthPitch: 0 };
            let hr = d3d_device_context.Map(surface as *mut ID3D11Resource, 0, d3d11::D3D11_MAP_READ, 0, &mut mapped_resource);
            if hr == winerror::S_OK {
                let data = slice::from_raw_parts(mapped_resource.pData as *const u8, (height * mapped_resource.RowPitch) as usize);
                let mut mat = core::Mat::new_rows_cols_with_data(height, width, core::CV_8UC4, data, mapped_resource.RowPitch as usize)?;
                cvt_color(&mat, &mut mat, COLOR_BGRA2BGR, 0)?;


                captured_mat = Some(mat);
                d3d_device_context.Unmap(surface as *mut ID3D11Resource, 0);
            }

            frame.Close()?;
        }
        Ok(())
    });

    let capture_session = GraphicsCaptureSession::new()?;
    capture_session.Initialize(&capture_item)?;
    capture_session.StartCapture()?;

    std::thread::sleep(Duration::from_millis(100));

    capture_session.Close()?;
    frame_pool.RemoveFrameArrived(&frame_arrived_token)?;

    match captured_mat {
        Some(mat) => Ok(mat),
        None => Err("Failed to capture the window content".into()),
    }
}



fn main() {
    highgui::named_window("Image", highgui::WINDOW_AUTOSIZE).unwrap();

    if let Some(hwnd) = find_window("Blender") {
        loop {
            let img = match capture_window(hwnd) {
                Ok(img) => img,
                Err(e) => {
                    eprintln!("Error: {}", e);
                    continue;
                }
            };

            let size = img.size().unwrap();
            if size.width > 0 && size.height > 0 {
                let cloned_img = img.clone();
                highgui::imshow("Image", &cloned_img).unwrap();
            }

            let key = highgui::wait_key(10).unwrap();
            if key == 27 { // ESC key
                break;
            }
        }
    } else {
        eprintln!("Could not find the Blender window.");
    }
}


