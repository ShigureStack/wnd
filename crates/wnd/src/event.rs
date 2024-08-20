use crate::{
    driver::{runner::ReturnCode, EventRunner},
    window::{Window, WindowResult},
};

pub struct Context {
    pub(crate) runner: EventRunner,
}

impl Context {
    pub fn new() -> Self {
        Self {
            runner: EventRunner::new(),
        }
    }

    pub fn create_window(&self) -> WindowResult<Window> {
        Window::new(self)
    }
}

pub enum Event {}

pub struct EventDispatcher {
    context: Context,
}

impl EventDispatcher {
    pub fn new() -> Self {
        Self {
            context: Context::new(),
        }
    }

    pub fn with_handler<T: EventHandler>(&self, mut handler: T) {
        handler.init(&self.context);
        self.context.runner.register_handler(move |e| match e {})
    }

    pub fn dispatch(&self) -> Option<ReturnCode> {
        self.context.runner.dispatch_events()
    }
}

pub trait EventHandler {
    fn init(&mut self, context: &Context);
    fn window_event(&mut self, context: &Context, window: &Window, event: Event);
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn event_loop() {
        let dispatcher = EventDispatcher::new();

        #[derive(Default)]
        struct App {
            window: Option<Window>,
        }

        impl EventHandler for App {
            fn init(&mut self, context: &Context) {
                let window = context.create_window().expect("unable to create window");
                window.apply_system_appearance();
                self.window = Some(window);
            }
            fn window_event(&mut self, context: &Context, window: &Window, event: Event) {}
        }

        dispatcher.with_handler(App::default());

        loop {
            match dispatcher.dispatch() {
                Some(code) => match code {
                    ReturnCode::Exit => break,
                },
                _ => {}
            }
        }
    }
}
