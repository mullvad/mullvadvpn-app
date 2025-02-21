use std::cell::RefCell;
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
use cacao::text::{AttributedString, Label};
use cacao::view::View;
use installer_downloader::delegate::ErrorMessage;
use objc_id::Id;

use crate::resource::{
    BANNER_DESC, BETA_LINK_TEXT, BETA_PREFACE_DESC, CANCEL_BUTTON_TEXT, DOWNLOAD_BUTTON_TEXT,
    WINDOW_HEIGHT, WINDOW_TITLE, WINDOW_WIDTH,
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
        unsafe { Id::from_ptr(msg_send![class!(NSColor), colorWithRed:r green:g blue:b alpha:a]) };
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
        delegate.inner.borrow().layout();
    }

    fn should_terminate_after_last_window_closed(&self) -> bool {
        true
    }
}

/// Dispatcher actions
pub enum Action {
    /// User clicked the download button
    DownloadClick(Arc<Mutex<Box<dyn Fn() + Send>>>),
    /// User clicked the cancel button
    CancelClick(Arc<Mutex<Box<dyn Fn() + Send>>>),
    /// Run callback on main thread
    QueueMain(Mutex<Option<Box<dyn for<'a> FnOnce(&'a mut AppWindow) + Send>>>),
    /// User clicked the retry button in the error view
    ErrorRetry(Arc<Mutex<Box<dyn Fn() + Send>>>),
    /// User clicked the cancel button in the error view
    ErrorCancel(Arc<Mutex<Box<dyn Fn() + Send>>>),
    /// Quit the application.
    Quit,
}

impl Dispatcher for AppImpl {
    type Message = Action;

    fn on_ui_message(&self, message: Self::Message) {
        let delegate = self.window.delegate.as_ref().unwrap();
        match message {
            Action::DownloadClick(cb) => {
                let cb = cb.lock().unwrap();
                cb();
            }
            Action::CancelClick(cb) => {
                let cb = cb.lock().unwrap();
                cb();
            }
            Action::QueueMain(cb) => {
                // NOTE: We assume that this won't panic because they will never run simultaneously
                let mut borrowed = delegate.inner.borrow_mut();
                let cb = cb.lock().unwrap().take().unwrap();
                cb(&mut borrowed);
            }
            Action::ErrorRetry(cb) => {
                let cb = cb.lock().unwrap();
                cb();
            }
            Action::ErrorCancel(cb) => {
                let cb = cb.lock().unwrap();
                cb();
            }
            Action::Quit => {
                self.window.close();
            }
        }
    }

    fn on_background_message(&self, _message: Self::Message) {
        // TODO
    }
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

    pub error_view: Option<ErrorView>,
    pub error_retry_callback: Option<Arc<Mutex<Box<dyn Fn() + Send + 'static>>>>,
    pub error_cancel_callback: Option<Arc<Mutex<Box<dyn Fn() + Send + 'static>>>>,

    pub download_text: Label,

    pub beta_link_preface: Label,
    pub beta_link: Label,
}

pub struct ErrorView {
    pub view: View,
    pub text: Label,
    pub circle: ImageView,
    pub retry_button: Button,
    pub cancel_button: Button,
}

pub struct DownloadButton {
    pub button: Button,
}

impl Default for DownloadButton {
    fn default() -> Self {
        let button = Button::new(DOWNLOAD_BUTTON_TEXT);
        Self { button }
    }
}

pub struct CancelButton {
    pub button: Button,
}

impl Default for CancelButton {
    fn default() -> Self {
        let button = Button::new(CANCEL_BUTTON_TEXT);
        Self { button }
    }
}

impl AppWindow {
    pub fn layout(&self) {
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

        let mut attr_text = AttributedString::new(&BETA_LINK_TEXT);
        attr_text.set_text_color(Color::Link, 0..BETA_LINK_TEXT.len() as isize);

        self.beta_link.set_attributed_text(attr_text);
        self.main_view.add_subview(&self.beta_link);

        LayoutConstraint::activate(&[
            self.download_text
                .top
                .constraint_equal_to(&self.status_text.bottom)
                .offset(16.),
            self.download_text
                .center_x
                .constraint_equal_to(&self.main_view.center_x),
            self.download_button
                .button
                .center_x
                .constraint_equal_to(&self.main_view.center_x),
            self.download_button
                .button
                .top
                .constraint_equal_to(&self.status_text.bottom)
                .offset(16.),
            self.progress
                .top
                .constraint_equal_to(&self.download_button.button.top)
                .offset(32.),
            self.progress
                .left
                .constraint_equal_to(&self.main_view.left)
                .offset(30.),
            self.progress
                .right
                .constraint_equal_to(&self.main_view.right)
                .offset(-30.),
            self.progress.height.constraint_equal_to_constant(16.0f64),
            self.cancel_button
                .button
                .center_x
                .constraint_equal_to(&self.main_view.center_x),
            self.cancel_button
                .button
                .top
                .constraint_equal_to(&self.progress.bottom)
                .offset(16.),
            self.beta_link_preface
                .bottom
                .constraint_equal_to(&self.main_view.bottom)
                .offset(-24.),
            self.beta_link_preface
                .left
                .constraint_equal_to(&self.main_view.left)
                .offset(24.),
            self.beta_link
                .bottom
                .constraint_equal_to(&self.beta_link_preface.bottom),
            self.beta_link
                .left
                .constraint_equal_to(&self.beta_link_preface.right),
        ]);
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
    pub fn new(
        main_view: &View,
        message: ErrorMessage,
        on_retry: Option<impl Fn() + Send + Sync + 'static>,
        on_cancel: Option<impl Fn() + Send + Sync + 'static>,
    ) -> Self {
        let mut error_view = ErrorView {
            view: Default::default(),
            text: Default::default(),
            circle: Default::default(),
            retry_button: Button::new(&message.retry_button_text),
            cancel_button: Button::new(&message.cancel_button_text),
        };

        let ErrorView {
            view,
            text,
            circle,
            retry_button,
            cancel_button,
        } = &mut error_view;

        text.set_text(message.status_text);
        circle.set_image(&ALERT_CIRCLE);

        if let Some(on_cancel) = on_cancel {
            cancel_button.set_action(on_cancel);
        }
        if let Some(on_retry) = on_retry {
            retry_button.set_action(on_retry);
        }

        view.add_subview(text);
        view.add_subview(circle);
        main_view.add_subview(view);
        main_view.add_subview(retry_button);
        main_view.add_subview(cancel_button);

        LayoutConstraint::activate(&[
            view.center_x.constraint_equal_to(&main_view.center_x),
            view.center_y
                .constraint_equal_to(&main_view.top)
                .offset(74.),
            view.width.constraint_equal_to_constant(536.),
            text.center_y.constraint_equal_to(&view.center_y),
            text.left.constraint_equal_to(&circle.right).offset(16.),
            text.right.constraint_equal_to(&view.right),
            circle.left.constraint_equal_to(&view.left),
            circle.center_y.constraint_equal_to(&text.center_y),
            retry_button
                .top
                .constraint_equal_to(&text.bottom)
                .offset(24.),
            cancel_button
                .top
                .constraint_equal_to(&text.bottom)
                .offset(24.),
            retry_button
                .left
                .constraint_equal_to(&view.center_x)
                .offset(8.),
            cancel_button
                .right
                .constraint_equal_to(&view.center_x)
                .offset(-8.),
            retry_button.width.constraint_equal_to_constant(213.),
            cancel_button.width.constraint_equal_to_constant(213.),
        ]);

        error_view
    }
}

impl Drop for ErrorView {
    fn drop(&mut self) {
        let ErrorView {
            view,
            text,
            circle,
            retry_button,
            cancel_button,
        } = self;
        view.remove_from_superview();
        text.remove_from_superview();
        circle.remove_from_superview();
        retry_button.remove_from_superview();
        cancel_button.remove_from_superview();
    }
}
