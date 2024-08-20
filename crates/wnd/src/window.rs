use std::sync::Arc;
use raw_window_handle::{HandleError, HasWindowHandle, WindowHandle};

use crate::driver::{self, error::WindowHandlerError, EventRunner};
use crate::event::Context;

pub struct Window {
    handler: driver::WindowHandler,
}

#[derive(Debug)]
pub enum WindowError {
    WindowHandlerError(WindowHandlerError),
}

pub type WindowResult<T> = Result<T, WindowError>;

impl Window {
    pub fn new(runner: Arc<EventRunner>) -> WindowResult<Self> {
        let handler = match driver::WindowHandler::new(runner) {
            Ok(handler) => handler,
            Err(err) => return Err(WindowError::WindowHandlerError(err)),
        };

        Ok(Self { handler })
    }

    pub fn apply_system_appearance(&self) {
        self.handler.apply_system_appearance();
    }

    pub fn set_title(&self, title: &str) {
        self.handler.set_title(title);
    }

    pub fn get_title(&self) {
        self.handler.get_title()
    }
}

impl HasWindowHandle for Window {
    fn window_handle(&self) -> Result<WindowHandle<'_>, HandleError> {
        let raw = self.handler.rwh()?;

        Ok(unsafe { WindowHandle::borrow_raw(raw) })
    }
}
