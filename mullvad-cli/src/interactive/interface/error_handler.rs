use super::{app::AppActions, dialog::Dialog};
use crate::interactive::component::{Component, Frame};

use crossterm::event::{Event, KeyCode, KeyEvent};
use parking_lot::Mutex;
use std::sync::Arc;
use tui::layout::Rect;

pub struct ErrorMessage {
    pub message: String,
    pub ignorable: bool,
}

pub struct ErrorHandler<T: Component> {
    actions: AppActions,
    error: Arc<Mutex<Option<ErrorMessage>>>,
    child: T,
}

impl<T: Component> ErrorHandler<T> {
    pub fn new(actions: AppActions, receiver: flume::Receiver<ErrorMessage>, child: T) -> Self {
        let new_error_handler = Self {
            actions,
            error: Arc::new(Mutex::new(None)),
            child,
        };

        new_error_handler.listen_error_messages(receiver);
        new_error_handler
    }

    fn listen_error_messages(&self, receiver: flume::Receiver<ErrorMessage>) {
        let actions = self.actions.clone();
        let error = self.error.clone();
        tokio::spawn(async move {
            while let Ok(new_error) = receiver.recv_async().await {
                {
                    let mut error = error.lock();
                    *error = Some(new_error);
                }
                actions.redraw_async().await;
            }
        });
    }
}

impl<T: Component> Component for ErrorHandler<T> {
    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) {
        self.child.draw(f, area);

        if let Some(ref error) = *self.error.lock() {
            Dialog::new("An error occured", &error.message).draw(f, area);
        }
    }

    fn handle_event(&mut self, event: Event) {
        let mut error = self.error.lock();
        if let Some(ErrorMessage { ignorable, .. }) = *error {
            if ignorable && event == Event::Key(KeyEvent::from(KeyCode::Esc)) {
                *error = None;
                self.actions.redraw();
            }
        } else {
            self.child.handle_event(event);
        }
    }
}
