use std::{borrow::BorrowMut, cell::Cell, mem::transmute};

use windows::Win32::{
    Foundation::{HWND, LPARAM, LRESULT, WPARAM},
    Graphics::Gdi::UpdateWindow,
    System::LibraryLoader::GetModuleHandleW,
    UI::{
        HiDpi::{
            GetDpiForWindow, SetProcessDpiAwarenessContext,
            DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2,
        },
        WindowsAndMessaging::{
            CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, LoadCursorW,
            PeekMessageW, PostQuitMessage, RegisterClassW, SetWindowPos, ShowWindow,
            TranslateMessage, CS_HREDRAW, CS_VREDRAW, IDI_APPLICATION, MSG, PM_REMOVE,
            SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOZORDER, SW_SHOW, WINDOW_EX_STYLE, WM_DESTROY,
            WM_QUIT, WNDCLASSW, WS_OVERLAPPEDWINDOW,
        },
    },
};

use crate::{
    driver::{
        error::{CreateWindowError, WindowHandlerError, WindowHandlerResult},
        runner::ReturnCode,
        win32::utils::string::StringExt,
    },
    event::Event,
};

pub struct NativeWindow {
    hwnd: HWND,
}

impl NativeWindow {
    pub fn new() -> WindowHandlerResult<Self> {
        let hwnd = match Self::create_window() {
            Ok(hwnd) => hwnd,
            Err(err) => return Err(WindowHandlerError::CreateWindowError(err)),
        };

        Ok(Self { hwnd })
    }

    unsafe extern "system" fn wndproc(
        hwnd: HWND,
        u_msg: u32,
        w_param: WPARAM,
        l_param: LPARAM,
    ) -> LRESULT {
        match u_msg {
            WM_DESTROY => {
                PostQuitMessage(0);
                return LRESULT(0);
            }
            _ => DefWindowProcW(hwnd, u_msg, w_param, l_param),
        }
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
                (100 as f32 * dpi / 96.0) as i32,
                (100 as f32 * dpi / 96.0) as i32,
                SWP_NOMOVE | SWP_NOZORDER | SWP_NOACTIVATE,
            )
        } {
            Err(..) => return Err(CreateWindowError::UnableToEnableHiDpiSupport),
            _ => {}
        }

        let _ = unsafe { ShowWindow(hwnd, SW_SHOW) };
        let _ = unsafe { UpdateWindow(hwnd) };

        Ok(hwnd)
    }

    pub fn set_title(&self, title: &str) {}

    pub fn get_title(&self) {}
}

pub(crate) struct EventRunner {
    handler: Cell<Option<Box<dyn FnMut(Event) -> ()>>>,
}

impl EventRunner {
    pub fn new() -> Self {
        Self::enable_hidpi_support();

        Self {
            handler: Cell::new(None),
        }
    }

    fn enable_hidpi_support() {
        unsafe { SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2) };
    }

    pub fn register_handler<F: FnMut(Event) -> ()>(&self, handler: F) {
        // Erase lifetime
        let handler =
            unsafe { transmute::<Box<dyn FnMut(Event)>, Box<dyn FnMut(Event)>>(Box::new(handler)) };
        // Resetting an event handler without before clearing is prohibited.
        assert!(self.handler.replace(Some(handler)).is_none());
    }

    pub fn dispatch_events(&self) -> Option<ReturnCode> {
        let mut msg = MSG::default();

        unsafe {
            if PeekMessageW(msg.borrow_mut(), None, 0, 0, PM_REMOVE).as_bool() {
                let _ = TranslateMessage(msg.borrow_mut());
                DispatchMessageW(msg.borrow_mut());

                if msg.message == WM_QUIT {
                    return Some(ReturnCode::Exit);
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn create_window() {
        assert!(NativeWindow::new().is_ok());
    }
}
