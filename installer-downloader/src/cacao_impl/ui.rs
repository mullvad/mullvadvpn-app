use std::cell::RefCell;
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, LazyLock, Mutex, RwLock};

use cacao::appkit::window::{Window, WindowConfig, WindowDelegate};
use cacao::appkit::{App, AppDelegate};
use cacao::button::Button;
use cacao::color::Color;
use cacao::image::{Image, ImageView};
use cacao::layout::{Layout, LayoutConstraint};
use cacao::notification_center::Dispatcher;
use cacao::objc::{class, msg_send, sel, sel_impl};
use cacao::progress::ProgressIndicator;
use cacao::text::Label;
use cacao::view::View;
use objc_id::Id;

use crate::resource::{
    BANNER_DESC, BETA_LINK_TEXT, BETA_PREFACE_DESC, CANCEL_BUTTON_TEXT, DOWNLOAD_BUTTON_TEXT,
    RETRY_BUTTON_TEXT, STABLE_LINK_TEXT, WINDOW_HEIGHT, WINDOW_TITLE, WINDOW_WIDTH,
};

/// Logo render in the banner
const LOGO_IMAGE_DATA: &[u8] = include_bytes!("../../assets/logo-icon.svg");

/// Logo banner text
const LOGO_TEXT_DATA: &[u8] = include_bytes!("../../assets/logo-text.svg");

const ALERT_CIRCLE_IMAGE_DATA: &[u8] = include_bytes!("../../assets/alert-circle.svg");

/// Banner background color: #192e45
static BANNER_COLOR: LazyLock<Color> = LazyLock::new(|| {
    let r = 0x19 as f64 / 255.;
    let g = 0x2e as f64 / 255.;
    let b = 0x45 as f64 / 255.;
    let a = 1.;

    // NOTE: colorWithCalibratedRed is used by cacao by default, but it renders a different color
    //       than it does for background color of the image. I believe this is because the
    //       calibrated uses the current color profile.
    //       Maybe using calibrated colors is more correct? Rendering different colors *definitely*
    //       is not.
    let id =
        // SAFETY: This function returns a pointer to a refcounted NSColor instance, and panics if
        //         a null pointer is passed.
        // See https://developer.apple.com/documentation/appkit/nscolor/init(red:green:blue:alpha:)?language=objc
        unsafe { Id::from_retained_ptr(msg_send![class!(NSColor), colorWithRed:r green:g blue:b alpha:a]) };
    Color::Custom(Arc::new(RwLock::new(id)))
});

static LOGO: LazyLock<Image> = LazyLock::new(|| Image::with_data(LOGO_IMAGE_DATA));
static LOGO_TEXT: LazyLock<Image> = LazyLock::new(|| Image::with_data(LOGO_TEXT_DATA));
static ALERT_CIRCLE: LazyLock<Image> = LazyLock::new(|| Image::with_data(ALERT_CIRCLE_IMAGE_DATA));

pub struct AppImpl {
    window: Window<AppWindowWrapper>,
}

impl Default for AppImpl {
    fn default() -> Self {
        Self {
            window: Window::with(WindowConfig::default(), AppWindowWrapper::default()),
        }
    }
}

impl AppDelegate for AppImpl {
    fn did_finish_launching(&self) {
        App::activate();

        self.window.show();

        let delegate = self.window.delegate.as_ref().unwrap();
        delegate.inner.borrow_mut().layout();
    }

    fn should_terminate_after_last_window_closed(&self) -> bool {
        true
    }
}

/// Dispatcher actions
pub enum Action {
    /// User clicked a button.
    ButtonClick {
        /// The callback to be invoked in the main thread.
        callback: Arc<Mutex<Box<dyn Fn() + Send>>>,
    },
    /// Run callback on main thread
    QueueMain(Mutex<Option<MainThreadCallback>>),
    /// Quit the application.
    Quit,
}

/// Callback used for `QueueMain`
pub type MainThreadCallback = Box<dyn for<'a> FnOnce(&'a mut AppWindow) + Send>;

impl Dispatcher for AppImpl {
    type Message = Action;

    fn on_ui_message(&self, message: Self::Message) {
        let delegate = self.window.delegate.as_ref().unwrap();
        match message {
            Action::ButtonClick { callback } => {
                let callback = callback.lock().unwrap();
                callback();
            }
            Action::QueueMain(cb) => {
                let mut borrowed = delegate.inner.borrow_mut();
                let cb = cb.lock().unwrap().take().unwrap();
                cb(&mut borrowed);
            }
            Action::Quit => {
                self.window.close();
            }
        }
    }

    fn on_background_message(&self, _message: Self::Message) {}
}

#[derive(Default)]
pub struct AppWindowWrapper {
    pub inner: RefCell<AppWindow>,
}

#[derive(Default)]
pub struct AppWindow {
    pub content: View,

    pub banner: View,
    pub banner_logo_view: ImageView,
    pub banner_logo_text_view: ImageView,
    pub banner_desc: Label,

    pub main_view: View,

    pub download_button: DownloadButton,
    pub cancel_button: CancelButton,

    pub progress: ProgressIndicator,

    pub status_text: Label,
    /// The y position constraint of [Self::status_text].
    /// This exists because we need to shift it up when download_text is revealed.
    pub status_text_position_y: Option<LayoutConstraint>,

    pub error_view: ErrorView,
    pub error_retry_callback: Option<Arc<Mutex<ErrorViewClickCallback>>>,
    pub error_cancel_callback: Option<Arc<Mutex<ErrorViewClickCallback>>>,

    pub download_text: Label,

    pub beta_link_preface: Label,
    pub beta_link: LinkToBeta,

    pub stable_link: LinkToStable,
}

#[derive(Default)]
pub struct ErrorView {
    pub view: View,
    pub text: Label,
    pub circle: ImageView,
    pub retry_button: RetryButton,
    pub cancel_button: CancelButton,
}

pub type ErrorViewClickCallback = Box<dyn Fn() + Send>;

/// Create a Button newtype that impls Default
macro_rules! button_wrapper {
    ($name:ident, $text:expr) => {
        pub struct $name {
            pub button: ::cacao::button::Button,
        }

        impl Default for $name {
            fn default() -> Self {
                Self {
                    button: Button::new(&$text),
                }
            }
        }

        impl Deref for $name {
            type Target = ::cacao::button::Button;
            fn deref(&self) -> &Self::Target {
                &self.button
            }
        }

        impl DerefMut for $name {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.button
            }
        }

        impl $name {
            /// Register a callback to be executed on the main thread when this button is pressed.
            #[allow(dead_code)]
            pub fn set_callback(&mut self, callback: impl Fn() + Send + 'static) {
                // Wrap it in an Arc<Mutex> to make it Sync.
                // We need this because Dispatcher demands sync, but the AppDelegate trait does not
                // impose that requirement on the callback.
                let callback = Box::new(callback) as Box<dyn Fn() + Send>;
                let callback = Arc::new(Mutex::new(callback));
                self.button.set_action(move || {
                    let callback = callback.clone();
                    let callback = Action::ButtonClick { callback };
                    cacao::appkit::App::<super::ui::AppImpl, _>::dispatch_main(callback);
                });
            }
        }
    };
}

button_wrapper!(LinkToBeta, BETA_LINK_TEXT);
button_wrapper!(LinkToStable, format!("← {STABLE_LINK_TEXT}"));
button_wrapper!(DownloadButton, DOWNLOAD_BUTTON_TEXT);
button_wrapper!(CancelButton, CANCEL_BUTTON_TEXT);
button_wrapper!(RetryButton, RETRY_BUTTON_TEXT);

impl AppWindow {
    pub fn layout(&mut self) {
        self.banner_logo_view.set_image(&LOGO);
        self.banner_logo_text_view.set_image(&LOGO_TEXT);
        self.banner.set_background_color(&*BANNER_COLOR);

        self.banner.add_subview(&self.banner_logo_view);
        self.banner.add_subview(&self.banner_logo_text_view);

        self.content.add_subview(&self.banner);
        self.content.add_subview(&self.main_view);

        self.main_view.add_subview(&self.progress);
        self.progress.set_hidden(true);
        self.progress.set_indeterminate(false);

        self.banner_desc.set_text(BANNER_DESC);
        self.banner_desc.set_text_color(Color::SystemWhite);
        self.banner.add_subview(&self.banner_desc);
        self.banner_desc
            .set_line_break_mode(cacao::text::LineBreakMode::WrapWords);

        LayoutConstraint::activate(&[
            self.banner_logo_view
                .bottom
                .constraint_equal_to(&self.banner_desc.top)
                .offset(-8.),
            self.banner_logo_view
                .left
                .constraint_equal_to(&self.banner.left)
                .offset(24.),
            self.banner_logo_view
                .width
                .constraint_equal_to_constant(32.0f64),
            self.banner_logo_view
                .height
                .constraint_equal_to_constant(32.0f64),
            self.banner_desc
                .left
                .constraint_equal_to(&self.banner_logo_view.left),
            self.banner_desc
                .bottom
                .constraint_equal_to(&self.banner.bottom)
                .offset(-16.),
            self.banner_desc
                .right
                .constraint_equal_to(&self.banner.right)
                .offset(-24.),
        ]);
        LayoutConstraint::activate(&[
            self.banner_logo_text_view
                .top
                .constraint_equal_to(&self.banner_logo_view.top)
                .offset(9.4),
            self.banner_logo_text_view
                .left
                .constraint_equal_to(&self.banner_logo_view.right)
                .offset(12.),
            self.banner_logo_text_view
                .width
                .constraint_equal_to_constant(122.),
            self.banner_logo_text_view
                .height
                .constraint_equal_to_constant(13.),
        ]);

        LayoutConstraint::activate(&[
            self.banner.left.constraint_equal_to(&self.content.left),
            self.banner.right.constraint_equal_to(&self.content.right),
            self.banner.top.constraint_equal_to(&self.content.top),
            self.banner.height.constraint_equal_to_constant(122.),
        ]);

        LayoutConstraint::activate(&[
            self.main_view.left.constraint_equal_to(&self.content.left),
            self.main_view
                .right
                .constraint_equal_to(&self.content.right),
            self.main_view.top.constraint_equal_to(&self.banner.bottom),
            self.main_view
                .bottom
                .constraint_equal_to(&self.content.bottom),
        ]);

        self.main_view.add_subview(&self.status_text);
        self.main_view.add_subview(&self.download_text);
        self.main_view.add_subview(&self.download_button.button);
        self.main_view.add_subview(&self.cancel_button.button);

        self.beta_link_preface.set_text(BETA_PREFACE_DESC);
        self.main_view.add_subview(&self.beta_link_preface);

        self.beta_link.set_text_color(Color::Link);
        self.beta_link.set_bordered(false);
        self.main_view.add_subview(&*self.beta_link);

        self.stable_link.set_text_color(Color::Link);
        self.stable_link.set_bordered(false);
        self.main_view.add_subview(&*self.stable_link);

        let status_text_position_y = self.status_text_position_y.get_or_insert_with(|| {
            self.status_text
                .top
                .constraint_equal_to(&self.main_view.top)
                .offset(59.)
        });

        self.error_view.layout(&self.main_view);

        LayoutConstraint::activate(&[
            status_text_position_y.clone(),
            self.status_text
                .center_x
                .constraint_equal_to(&self.main_view.center_x),
            self.download_text
                .top
                .constraint_equal_to(&self.status_text.bottom)
                .offset(4.),
            self.download_text
                .center_x
                .constraint_equal_to(&self.main_view.center_x),
            self.download_button
                .button
                .center_x
                .constraint_equal_to(&self.main_view.center_x),
            self.download_button
                .button
                .center_y
                .constraint_equal_to(&self.main_view.center_y),
            self.download_button
                .button
                .width
                .constraint_equal_to_constant(213.),
            self.download_button
                .button
                .height
                .constraint_equal_to_constant(22.),
            self.progress
                .top
                .constraint_equal_to(&self.download_text.bottom),
            self.progress
                .left
                .constraint_equal_to(&self.main_view.left)
                .offset(30.),
            self.progress
                .right
                .constraint_equal_to(&self.main_view.right)
                .offset(-30.),
            self.progress.height.constraint_equal_to_constant(36.),
            self.cancel_button
                .button
                .center_x
                .constraint_equal_to(&self.main_view.center_x),
            self.cancel_button
                .button
                .top
                .constraint_equal_to(&self.progress.bottom),
            self.cancel_button
                .button
                .width
                .constraint_equal_to_constant(213.),
            self.cancel_button
                .button
                .height
                .constraint_equal_to_constant(22.),
            self.beta_link_preface
                .bottom
                .constraint_equal_to(&self.main_view.bottom)
                .offset(-24.),
            self.beta_link_preface
                .left
                .constraint_equal_to(&self.main_view.left)
                .offset(24.),
            self.beta_link
                .center_y
                .constraint_equal_to(&self.beta_link_preface.center_y),
            self.beta_link
                .left
                .constraint_equal_to(&self.beta_link_preface.right),
            self.stable_link
                .left
                .constraint_equal_to(&self.beta_link_preface.left),
            self.stable_link
                .center_y
                .constraint_equal_to(&self.beta_link_preface.center_y),
        ]);
    }

    // If there is a download_text, move status_text up to make room
    pub fn readjust_status_text(&mut self) {
        let text = self.download_text.get_text();

        let offset = if text.is_empty() { 59.0 } else { 39.0 };

        if let Some(previous_constraint) = self.status_text_position_y.take() {
            LayoutConstraint::deactivate(&[previous_constraint]);
        }

        let new_constraint = self
            .status_text
            .top
            .constraint_equal_to(&self.main_view.top)
            .offset(offset);
        self.status_text_position_y = Some(new_constraint.clone());
        LayoutConstraint::activate(&[new_constraint]);
    }
}

impl WindowDelegate for AppWindowWrapper {
    const NAME: &'static str = "MullvadInstallerDelegate";

    fn did_load(&mut self, window: Window) {
        window.set_title(WINDOW_TITLE);
        window.set_minimum_content_size(WINDOW_WIDTH as f64, WINDOW_HEIGHT as f64);
        window.set_maximum_content_size(WINDOW_WIDTH as f64, WINDOW_HEIGHT as f64);
        window.set_content_size(WINDOW_WIDTH as f64, WINDOW_HEIGHT as f64);
        window.set_content_view(&self.inner.borrow().content);
    }
}

impl ErrorView {
    pub fn layout(&mut self, main_view: &View) {
        self.circle.set_image(&ALERT_CIRCLE);

        self.view.add_subview(&self.text);
        self.view.add_subview(&self.circle);

        main_view.add_subview(&self.view);
        main_view.add_subview(&self.retry_button.button);
        main_view.add_subview(&self.cancel_button.button);

        self.retry_button.set_hidden(true);
        self.cancel_button.set_hidden(true);
        self.view.set_hidden(true);

        LayoutConstraint::activate(&[
            self.view.center_x.constraint_equal_to(&main_view.center_x),
            self.view
                .center_y
                .constraint_equal_to(&main_view.top)
                .offset(74.),
            self.view.width.constraint_equal_to_constant(536.),
            self.text.center_y.constraint_equal_to(&self.view.center_y),
            self.text
                .left
                .constraint_equal_to(&self.circle.right)
                .offset(16.),
            self.text.right.constraint_equal_to(&self.view.right),
            self.circle.left.constraint_equal_to(&self.view.left),
            self.circle
                .center_y
                .constraint_equal_to(&self.text.center_y),
            self.retry_button
                .button
                .top
                .constraint_equal_to(&self.text.bottom)
                .offset(24.),
            self.cancel_button
                .top
                .constraint_equal_to(&self.text.bottom)
                .offset(24.),
            self.retry_button
                .left
                .constraint_equal_to(&self.view.center_x)
                .offset(8.),
            self.cancel_button
                .button
                .right
                .constraint_equal_to(&self.view.center_x)
                .offset(-8.),
            self.retry_button
                .button
                .width
                .constraint_equal_to_constant(213.),
            self.cancel_button
                .button
                .width
                .constraint_equal_to_constant(213.),
        ]);
    }

    pub fn show(
        &mut self,
        status_text: String,
        on_retry: Option<impl Fn() + Send + Sync + 'static>,
        on_cancel: Option<impl Fn() + Send + Sync + 'static>,
    ) {
        self.text.set_text(status_text);

        if let Some(on_cancel) = on_cancel {
            self.cancel_button.set_action(on_cancel);
        } else {
            self.cancel_button.set_action(|| {});
        }
        if let Some(on_retry) = on_retry {
            self.retry_button.set_action(on_retry);
        } else {
            self.retry_button.set_action(|| {});
        }

        self.retry_button.set_hidden(false);
        self.cancel_button.set_hidden(false);
        self.view.set_hidden(false);
    }

    pub fn hide(&mut self) {
        self.retry_button.set_hidden(true);
        self.cancel_button.set_hidden(true);
        self.view.set_hidden(true);
    }
}
