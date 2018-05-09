extern crate app_dirs;

use app_dirs::AppInfo;

pub const PRODUCT_NAME: &str = "Mullvad VPN";

pub const APP_INFO: AppInfo = AppInfo {
    name: PRODUCT_NAME,
    author: "Mullvad",
};
