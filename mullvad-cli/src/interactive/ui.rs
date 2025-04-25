use std::time::Duration;

use super::component::{Backend, Component};

use crossterm::event::EventStream;
use futures_timer::Delay;
use tokio::select;
use tokio_stream::StreamExt;
use tui::Terminal;

#[derive(thiserror::Error, Debug)]
pub enum UiError {
    #[error("Failed to draw of component tree")]
    Draw(#[from] std::io::Error),

    #[error("Failed to send redraw message")]
    ReDraw(#[from] flume::SendError<UiMessage>),

    #[error("Failed to receive UiMessage")]
    MessageReceiver(#[from] flume::RecvError),
}

#[derive(Eq, PartialEq)]
pub enum UiMessage {
    Quit,
    Redraw,
}

pub async fn create<C, F>(terminal: &mut Terminal<Backend>, creator: F) -> Result<(), UiError>
where
    C: Component,
    F: FnOnce(flume::Sender<UiMessage>) -> C,
{
    let mut event_reader = EventStream::new();
    let (ui_sender, ui_receiver) = flume::unbounded();

    let mut root = creator(ui_sender.clone());
    ui_sender.send_async(UiMessage::Redraw).await?;

    loop {
        select! {
            event = event_reader.next() => {
                if let Some(Ok(event)) = event {
                    root.handle_event(event);
                }
            },

            // Delay recv by 8ms to make drawing cap at ~120 fps
            event = delayed_recv(&ui_receiver, 8) => match event? {
                UiMessage::Quit => break,
                UiMessage::Redraw => {
                    let drained = ui_receiver.drain();
                    if drained.len() > 0 && drained.into_iter().any(|message| message == UiMessage::Quit) {
                        break;
                    }

                    perform_draw(terminal, &mut root)?;
                }
            },
        };
    }

    Ok(())
}

async fn delayed_recv<T>(receiver: &flume::Receiver<T>, delay: u64) -> Result<T, flume::RecvError> {
    Delay::new(Duration::from_millis(delay)).await;
    receiver.recv_async().await
}

fn perform_draw(
    terminal: &mut Terminal<Backend>,
    root: &mut impl Component,
) -> Result<(), UiError> {
    terminal.draw(|f| root.draw(f, f.size()))?;
    Ok(())
}
