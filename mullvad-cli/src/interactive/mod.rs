use crossterm::{
    event::{DisableBracketedPaste, EnableBracketedPaste},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tui::{backend::CrosstermBackend, Terminal};

pub mod component;
pub mod interface;

mod ui;

pub(crate) async fn run() {
    enable_raw_mode().expect("Failed to setup interface");
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableBracketedPaste)
        .expect("Failed to setup interface");

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).expect("Failed to setup interface");

    ui::create(&mut terminal, interface::app::App::new)
        .await
        .expect("Failed to run ui");

    disable_raw_mode().expect("Failed to clean up");
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableBracketedPaste
    )
    .expect("Failed to clean up");
    terminal.show_cursor().expect("Failed to clean up");
}
