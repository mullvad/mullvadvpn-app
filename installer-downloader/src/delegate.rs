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

    /// Register click handler for the beta link
    fn on_beta_link<F>(&mut self, callback: F)
    where
        F: Fn() + Send + 'static;

    /// Register click handler for the stable link
    fn on_stable_link<F>(&mut self, callback: F)
    where
        F: Fn() + Send + 'static;

    /// Set status text
    fn set_status_text(&mut self, text: &str);

    /// Clear status text
    fn clear_status_text(&mut self);

    /// Set download text
    fn set_download_text(&mut self, text: &str);

    /// Clear download text
    fn clear_download_text(&mut self);

    /// Show download progress bar
    fn show_download_progress(&mut self);

    /// Hide download progress bar
    fn hide_download_progress(&mut self);

    /// Update download progress bar
    fn set_download_progress(&mut self, complete: u32);

    /// Clear download progress
    fn clear_download_progress(&mut self);

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

    /// Show stable text
    fn show_stable_text(&mut self);

    /// Hide stable text
    fn hide_stable_text(&mut self);

    /// Show error message
    fn show_error_message(&mut self, message: ErrorMessage);

    /// Hide error message
    fn hide_error_message(&mut self);

    /// Set error cancel callback
    fn on_error_message_retry<F>(&mut self, callback: F)
    where
        F: Fn() + Send + 'static;

    /// Set error cancel callback
    fn on_error_message_cancel<F>(&mut self, callback: F)
    where
        F: Fn() + Send + 'static;

    /// Exit the application
    fn quit(&mut self);

    /// Create queue for scheduling actions on UI (main) thread
    fn queue(&self) -> Self::Queue;
}

#[derive(Default, serde::Serialize)]
pub struct ErrorMessage {
    pub status_text: String,
    pub cancel_button_text: String,
    pub retry_button_text: String,
}

/// Schedules actions on the UI (main) thread from other threads
pub trait AppDelegateQueue<T: ?Sized>: Send + Clone {
    /// Schedule action on the UI (main) thread from other threads
    fn queue_main<F: FnOnce(&mut T) + 'static + Send>(&self, callback: F);
}
