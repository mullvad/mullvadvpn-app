#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use libui::prelude::*;

use crate::{controller, ui};

pub fn main() {
    let ui = UI::init().expect("Couldn't initialize UI library");

    let mut app_ui = ui::AppUi::new(&ui);

    let _app_controller = controller::AppController::new(&ui, &app_ui);

    app_ui.window.show();

    ui.main();
}
