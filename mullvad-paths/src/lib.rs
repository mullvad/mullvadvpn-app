extern crate app_dirs;
#[macro_use]
extern crate error_chain;

use app_dirs::AppInfo;

pub const PRODUCT_NAME: &str = "Mullvad VPN";

pub const APP_INFO: AppInfo = AppInfo {
    name: PRODUCT_NAME,
    author: "Mullvad",
};

error_chain! {
    errors {
        NoCacheDir { description("Unable to locate/create cache directory") }
    }
}

mod cache;
pub use cache::get_cache_dir;
