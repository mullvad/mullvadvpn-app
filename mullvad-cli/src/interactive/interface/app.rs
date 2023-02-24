use std::fmt::Display;

use super::{
    error_handler::{ErrorHandler, ErrorMessage},
    rpc_provider::RpcProvider,
    tunnel_state_provider::TunnelStateProvider,
    viewport::Viewport,
};
use crate::interactive::{
    component::{Component, Frame},
    ui::UiMessage,
};

use crossterm::event::{Event, KeyCode};
use tui::layout::Rect;

#[derive(Clone)]
pub struct AppActions {
    ui_sender: flume::Sender<UiMessage>,
    error_sender: flume::Sender<ErrorMessage>,
}

#[allow(dead_code)]
impl AppActions {
    pub fn redraw(&self) {
        self.handle_result(self.ui_sender.send(UiMessage::Redraw), false);
    }

    pub async fn redraw_async(&self) {
        self.handle_result_async(self.ui_sender.send_async(UiMessage::Redraw).await, false)
            .await;
    }

    fn error(&self, error: ErrorMessage) {
        self.error_sender
            .send(error)
            .expect("Failed to send error message");
    }

    async fn error_async(&self, error: ErrorMessage) {
        self.error_sender
            .send_async(error)
            .await
            .expect("Failed to send error message");
    }

    pub fn redraw_or_error<T, E: Display>(&self, result: Result<T, E>, ignorable: bool) {
        match result {
            Ok(_) => self.redraw(),
            Err(error) => self.error(ErrorMessage {
                message: error.to_string(),
                ignorable,
            }),
        }
    }

    pub async fn redraw_or_error_async<T, E: Display>(
        &self,
        result: Result<T, E>,
        ignorable: bool,
    ) {
        match result {
            Ok(_) => self.redraw_async().await,
            Err(error) => {
                self.error_async(ErrorMessage {
                    message: error.to_string(),
                    ignorable,
                })
                .await
            }
        }
    }

    pub fn handle_error<E: Display>(&self, error: E, ignorable: bool) {
        self.error(ErrorMessage {
            message: error.to_string(),
            ignorable,
        });
    }

    pub async fn handle_error_async<E: Display>(&self, error: E, ignorable: bool) {
        self.error_async(ErrorMessage {
            message: error.to_string(),
            ignorable,
        })
        .await;
    }

    pub fn handle_result<T, E: Display>(&self, result: Result<T, E>, ignorable: bool) {
        if let Err(error) = result {
            self.handle_error(error, ignorable);
        }
    }

    pub async fn handle_result_async<T, E: Display>(&self, result: Result<T, E>, ignorable: bool) {
        if let Err(error) = result {
            self.handle_error_async(error, ignorable).await;
        }
    }
}

pub struct App {
    ui_sender: flume::Sender<UiMessage>,
    actions: AppActions,
    error_handler: ErrorHandler<RpcProvider<TunnelStateProvider<Viewport>>>,
}

impl App {
    pub fn new(ui_sender: flume::Sender<UiMessage>) -> Self {
        let (error_sender, error_receiver) = flume::unbounded();
        let actions = AppActions {
            ui_sender: ui_sender.clone(),
            error_sender,
        };

        let cloned_actions = actions.clone();
        let rpc_provider = RpcProvider::new(|rpc| {
            let rpc_clone = rpc.clone();
            TunnelStateProvider::new(
                |tunnel_state_broadcast| {
                    Viewport::new(cloned_actions, rpc_clone, tunnel_state_broadcast)
                },
                rpc,
            )
        });

        let error_handler = ErrorHandler::new(actions.clone(), error_receiver, rpc_provider);

        Self {
            ui_sender,
            actions,
            error_handler,
        }
    }
}

impl Component for App {
    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) {
        self.error_handler.draw(f, area);
    }

    fn handle_event(&mut self, event: Event) {
        match event {
            Event::Key(event) if event.code == KeyCode::Char('q') => {
                let _ = self.ui_sender.send(UiMessage::Quit);
            }
            Event::Resize(_, _) => self.actions.redraw(),
            _ => self.error_handler.handle_event(event),
        }
    }
}
