//! Framework-agnostic module that hooks up a UI to actions

pub use crate::ui_downloader::UiProgressUpdater;

/// Trait implementing high-level UI actions
pub trait AppDelegate {
    /// Queue lets us perform actions from other threads
    type Queue: AppDelegateQueue<Self>;

    /// Register click handler for the download button
    fn on_download<F>(&mut self, callback: F)
    where
        F: Fn() + Send + 'static;

    /// Register click handler for the cancel button
    fn on_cancel<F>(&mut self, callback: F)
    where
        F: Fn() + Send + 'static;

    /// Set status text
    fn set_status_text(&mut self, text: &str);

    /// Set download text
    fn set_download_text(&mut self, text: &str);

    /// Show download progress bar
    fn show_download_progress(&mut self);

    /// Hide download progress bar
    fn hide_download_progress(&mut self);

    /// Update download progress bar
    fn set_download_progress(&mut self, complete: u32);

    /// Enable download button
    fn enable_download_button(&mut self);

    /// Disable download button
    fn disable_download_button(&mut self);

    /// Show download button
    fn show_download_button(&mut self);

    /// Hide download button
    fn hide_download_button(&mut self);

    /// Show cancel button
    fn show_cancel_button(&mut self);

    /// Hide cancel button
    fn hide_cancel_button(&mut self);

    /// Enable cancel button
    fn enable_cancel_button(&mut self);

    /// Disable cancel button
    fn disable_cancel_button(&mut self);

    /// Show beta text
    fn show_beta_text(&mut self);

    /// Hide beta text
    fn hide_beta_text(&mut self);

    /// Exit the application
    fn quit(&mut self);

    /// Create queue for scheduling actions on UI thread
    fn queue(&self) -> Self::Queue;
}

/// Schedules actions on the UI thread from other threads
pub trait AppDelegateQueue<T: ?Sized>: Send {
    fn queue_main<F: FnOnce(&mut T) + 'static + Send>(&self, callback: F);
}
