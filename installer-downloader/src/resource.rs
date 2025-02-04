//! Shared text and other resources

/// Window title
pub const WINDOW_TITLE: &str = "Mullvad VPN downloader";
/// Window width
pub const WINDOW_WIDTH: usize = 676;
/// Window height
pub const WINDOW_HEIGHT: usize = 390;

/// Text description in the top banner
pub const BANNER_DESC: &str = "The Mullvad VPN app installer will be downloaded from the nearest server and verified for authenticity";

/// Beta preface text
pub const BETA_PREFACE_DESC: &str = "Want to try the new Beta version? ";
/// Beta link text
pub const BETA_LINK_TEXT: &str = "Click here!";

/// Download button text
pub const DOWNLOAD_BUTTON_TEXT: &str = "Download & install";

/// Cancel button text
pub const CANCEL_BUTTON_TEXT: &str = "Cancel";

/// Displayed while fetching version info from the API
pub const FETCH_VERSION_DESC: &str = "Loading version details...";

/// The first part of "Version: 2025.1"
pub const LATEST_VERSION_PREFIX: &str = "Version";

/// Displayed while fetching version info from the API failed
pub const FETCH_VERSION_ERROR_DESC: &str = "Couldn't load version details, please try again or make sure you have the latest installer downloader.";

/// The first part of "Downloading from <some url>... (x%)", displayed during download
pub const DOWNLOADING_DESC_PREFIX: &str = "Downloading from";

/// Displayed after completed download
pub const DOWNLOAD_COMPLETE_DESC: &str = "Download complete. Verifying...";

/// Displayed after verification
pub const VERIFICATION_SUCCEEDED_DESC: &str = "Verification successful. Starting install...";
