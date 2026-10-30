// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use installer_downloader::controller::initialize_controller;
use installer_downloader::environment::Environment;
use installer_downloader::log;
use slint::Weak;

use crate::delegate::SlintApp;

mod delegate;

slint::include_modules!();

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _logger = log::init();
    let ui = AppWindow::new()?;
    let app = init_backend(ui.as_weak())?;
    ui.run()?;
    // Start working on message queue
    app.run().await?;
    Ok(())
}

/// [`delegate::App`] is responsible for hooking up the UI to their corresponsding action.
fn init_backend(ui: Weak<AppWindow>) -> anyhow::Result<SlintApp> {
    let environment = Environment::load()?;
    let mut delegate = delegate::SlintApp::new(ui);
    initialize_controller(&mut delegate, environment);
    Ok(delegate)
}
