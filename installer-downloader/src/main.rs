#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use libui::prelude::*;

mod app;
mod controller;
mod fetch;
mod ui;
mod verify;

fn main() {
    let ui = UI::init().expect("Couldn't initialize UI library");

    let mut app_ui = ui::AppUi::new(&ui);

    let _app_controller = controller::AppController::new(&ui, &app_ui);

    app_ui.window.show();

    ui.main();
}
