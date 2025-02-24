use native_windows_gui::{self as nwg, modal_fatal_message, ControlHandle};
use talpid_platform_metadata::get_native_arch;

use crate::delegate::{AppDelegate, AppDelegateQueue};

mod delegate;
mod ui;

pub fn main() {
    nwg::init().expect("Failed to init Native Windows GUI");
    nwg::Font::set_global_family("Segoe UI").expect("Failed to set default font");

    let window = ui::AppWindow::default();
    let window = window.layout().unwrap();

    let queue = window.borrow().queue();

    // Load "global" values and resources
    let Ok(env) = Environment::load() else {
        let parent = ControlHandle::NoHandle;
        let title = "Failed to initialize";
        let content = "Failed to detect CPU architecture";
        modal_fatal_message(parent, title, content)
    };

    queue.queue_main(|window| {
        crate::controller::initialize_controller(window);
    });

    nwg::dispatch_thread_events();
}

struct Environment {
    architecture: mullvad_update::format::Architecture,
}

impl Environment {
    pub fn load() -> Result<Self, ()> {
        let Some(architecture) = Self::get_arch().ok().flatten() else {
            return Err(());
        };

        Ok(Environment { architecture })
    }

    /// Try to map the host's CPU architecture to one of the CPU architectures the Mullvad VPN app
    /// supports.
    fn get_arch() -> Result<Option<VersionArchitecture>, std::io::Error> {
        match talpid_platform_metadata::get_native_arch()?? {
            talpid_platform_metadata::Architecture::X86 => VersionArchitecture::X86,
            talpid_platform_metadata::Architecture::Arm64 => VersionArchitecture::Arm64,
        }
    }
}
