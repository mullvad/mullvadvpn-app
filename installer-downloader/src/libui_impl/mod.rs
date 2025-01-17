use crate::controller;
use libui::prelude::*;

mod delegate;
mod ui;

pub fn main() {
    let ui = UI::init().expect("Couldn't initialize UI library");

    let mut app_ui = ui::AppUi::new(&ui);

    let mut app_delegate = delegate::LibuiAppDelegate::new(&ui, &app_ui);
    let _app_controller = crate::controller::initialize_controller(&mut app_delegate);

    app_ui.window.show();

    ui.main();
}
