use windows::Win32::{
    Foundation::{HWND, LPARAM, LRESULT, WPARAM},
    System::LibraryLoader::GetModuleHandleW,
    UI::{
        HiDpi::{
            GetDpiForWindow, SetProcessDpiAwarenessContext,
            DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2,
        },
        WindowsAndMessaging::{
            CreateWindowExW, DefWindowProcW, LoadCursorW, RegisterClassW, SetWindowPos, ShowWindow,
            CS_HREDRAW, CS_VREDRAW, IDI_APPLICATION, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOZORDER,
            SW_SHOW, WINDOW_EX_STYLE, WNDCLASSW, WS_OVERLAPPEDWINDOW,
        },
    },
};

use crate::driver::{
    error::{CreateWindowError, WindowHandlerError, WindowHandlerResult},
    win32::utils::string::ToUTF16String,
};

pub struct NativeWindow {
    hwnd: HWND,
}

impl NativeWindow {
    pub fn new() -> WindowHandlerResult<Self> {
        Self::enable_hidpi_support();
        let hwnd = match Self::create_window() {
            Ok(hwnd) => hwnd,
            Err(err) => return Err(WindowHandlerError::CreateWindowError(err)),
        };

        Ok(Self { hwnd })
    }

    fn enable_hidpi_support() {
        unsafe { SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2) };
    }

    unsafe extern "system" fn wndproc(
        hwnd: HWND,
        u_msg: u32,
        w_param: WPARAM,
        l_param: LPARAM,
    ) -> LRESULT {
        DefWindowProcW(hwnd, u_msg, w_param, l_param)
    }

    fn create_window() -> Result<HWND, CreateWindowError> {
        let classname = String::from("wndp").to_pcwstr();
        let hinstance = unsafe { GetModuleHandleW(None) }.unwrap();

        let class = unsafe {
            WNDCLASSW {
                style: CS_HREDRAW | CS_VREDRAW,
                lpfnWndProc: Some(Self::wndproc),
                hInstance: hinstance.into(),
                lpszClassName: classname,
                hCursor: LoadCursorW(None, IDI_APPLICATION).unwrap(),
                ..Default::default()
            }
        };

        unsafe { RegisterClassW(&class) };

        let hwnd = match unsafe {
            CreateWindowExW(
                WINDOW_EX_STYLE(0),
                classname,
                String::new().to_pcwstr(),
                WS_OVERLAPPEDWINDOW,
                0,
                0,
                0,
                0,
                None,
                None,
                hinstance,
                None,
            )
        } {
            Ok(hwnd) => hwnd,
            Err(..) => return Err(CreateWindowError::FailedToCreateWindow),
        };

        let dpi = unsafe { GetDpiForWindow(hwnd) as f32 };

        match unsafe {
            SetWindowPos(
                hwnd,
                None,
                0,
                0,
                (10 as f32 * dpi / 96.0) as i32,
                (10 as f32 * dpi / 96.0) as i32,
                SWP_NOMOVE | SWP_NOZORDER | SWP_NOACTIVATE,
            )
        } {
            Err(..) => return Err(CreateWindowError::UnableToEnableHiDpiSupport),
            _ => {}
        }

        let _ = unsafe { ShowWindow(hwnd, SW_SHOW) };

        Ok(hwnd)
    }

    pub fn set_title(&self, title: &str) {}

    pub fn get_title(&self) {}
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn create_window() {
        assert!(NativeWindow::new().is_ok());
    }

    #[test]
    fn window_loop() {}
}
