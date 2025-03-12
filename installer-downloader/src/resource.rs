//! Shared text and other resources

/// Installer downloader version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Window title
pub const WINDOW_TITLE: &str = "Mullvad VPN installer";
/// Window width
pub const WINDOW_WIDTH: usize = 600;
/// Window height
pub const WINDOW_HEIGHT: usize = 334;

/// Text description in the top banner
pub const BANNER_DESC: &str =
    "The Mullvad VPN app installer will be downloaded and verified for authenticity.";

/// Beta preface text
pub const BETA_PREFACE_DESC: &str = "Want to try the new Beta version? ";
/// Beta link text
pub const BETA_LINK_TEXT: &str = "Click here!";

/// Stable link text
pub const STABLE_LINK_TEXT: &str = "Back to stable version";

/// Dimensions of cancel button (including padding)
pub const CANCEL_BUTTON_SIZE: (usize, usize) = (150, 40);

/// Download button text
pub const DOWNLOAD_BUTTON_TEXT: &str = "Download & install";

/// Dimensions of download button (including padding)
pub const DOWNLOAD_BUTTON_SIZE: (usize, usize) = (150, 40);

/// Cancel button text
pub const CANCEL_BUTTON_TEXT: &str = "Cancel";

/// Displayed while fetching version info from the API
pub const FETCH_VERSION_DESC: &str = "Loading version details...";

/// The first part of "Version: 2025.1"
pub const LATEST_VERSION_PREFIX: &str = "Version";

/// Displayed while fetching version info from the API failed
pub const FETCH_VERSION_ERROR_DESC: &str = "Failed to load version details, please try again or make sure you have the latest installer downloader.";

/// Displayed while fetching version info from the API failed (retry button)
pub const FETCH_VERSION_ERROR_RETRY_BUTTON_TEXT: &str = "Try again";

/// Displayed while fetching version info from the API failed (cancel button)
pub const FETCH_VERSION_ERROR_CANCEL_BUTTON_TEXT: &str = "Cancel";

/// The first part of "Downloading from \<some url\>... (x%)", displayed during download
pub const DOWNLOADING_DESC_PREFIX: &str = "Downloading from";

/// Displayed after completed download
pub const DOWNLOAD_COMPLETE_DESC: &str = "Download complete. Verifying...";

/// Displayed when download fails
pub const DOWNLOAD_FAILED_DESC: &str = "Download failed, please check your internet connection or if you have enough space on your hard drive and try downloading again.";

/// Displayed when download fails (retry button)
pub const DOWNLOAD_FAILED_RETRY_BUTTON_TEXT: &str = "Try again";

/// Displayed when download fails (cancel button)
pub const DOWNLOAD_FAILED_CANCEL_BUTTON_TEXT: &str = "Cancel";

/// Displayed when download fails
pub const VERIFICATION_FAILED_DESC: &str = "Failed to verify download, please try downloading again or contact our support by sending an email to support@mullvadvpn.net with a description of what happened.";

/// Displayed when download fails (retry button)
pub const VERIFICATION_FAILED_RETRY_BUTTON_TEXT: &str = "Try again";

/// Displayed when download fails (cancel button)
pub const VERIFICATION_FAILED_CANCEL_BUTTON_TEXT: &str = "Cancel";

/// Displayed after verification
pub const VERIFICATION_SUCCEEDED_DESC: &str = "Verification successful. Starting install...";

/// Displayed when launch fails
pub const LAUNCH_FAILED_DESC: &str = "Failed to start installation, please try downloading again or contact our support by sending an email to support@mullvadvpn.net with a description of what happened.";

/// Displayed when launch fails (retry button)
pub const LAUNCH_FAILED_RETRY_BUTTON_TEXT: &str = "Try again";

/// Displayed when launch fails (cancel button)
pub const LAUNCH_FAILED_CANCEL_BUTTON_TEXT: &str = "Cancel";
