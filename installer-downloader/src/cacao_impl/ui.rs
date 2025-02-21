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
use objc_id::Id;

use crate::resource::{
    BANNER_DESC, BETA_LINK_TEXT, BETA_PREFACE_DESC, CANCEL_BUTTON_TEXT, DOWNLOAD_BUTTON_TEXT,
    WINDOW_HEIGHT, WINDOW_TITLE, WINDOW_WIDTH,
};

/// Logo render in the banner
const LOGO_IMAGE_DATA: &[u8] = include_bytes!("../logo-icon.svg");

/// Logo banner text
const LOGO_TEXT_DATA: &[u8] = include_bytes!("../logo-text.svg");

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
    pub progress: ProgressIndicator,
    pub text: Label,
    pub button: DownloadButton,
    pub cancel_button: CancelButton,

    pub beta_link_preface: Label,
    pub beta_link: Label,
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
        let button = Button::new("Cancel");
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

        self.banner_desc.set_text(BANNER_DESCRIPTION);
        self.banner_desc.set_text_color(Color::SystemWhite);
        self.banner.add_subview(&self.banner_desc);
        self.banner_desc
            .set_line_break_mode(cacao::text::LineBreakMode::WrapWords);

        let image_view_vert = self
            .banner_logo_view
            .top
            .constraint_equal_to(&self.banner.top)
            .offset(32. + 24.);
        LayoutConstraint::activate(&[
            image_view_vert,
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
                .top
                .constraint_equal_to(&self.banner_logo_view.bottom)
                .offset(8.),
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
            self.banner
                .height
                .constraint_less_than_or_equal_to_constant(160.),
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

        self.main_view.add_subview(&self.text);
        self.main_view.add_subview(&self.button.button);
        self.main_view.add_subview(&self.cancel_button.button);

        self.beta_link_preface.set_text(BETA_PREFACE_TEXT);
        self.main_view.add_subview(&self.beta_link_preface);

        let mut attr_text = AttributedString::new(&BETA_LINK_TEXT);
        attr_text.set_text_color(Color::Link, 0..BETA_LINK_TEXT.len() as isize);

        self.beta_link.set_attributed_text(attr_text);
        self.main_view.add_subview(&self.beta_link);

        LayoutConstraint::activate(&[
            self.text
                .top
                .constraint_greater_than_or_equal_to(&self.main_view.top)
                .offset(24.),
            self.text
                .center_x
                .constraint_equal_to(&self.main_view.center_x),
            self.button
                .button
                .center_x
                .constraint_equal_to(&self.main_view.center_x),
            self.button
                .button
                .top
                .constraint_equal_to(&self.text.bottom)
                .offset(16.),
            self.progress
                .top
                .constraint_equal_to(&self.button.button.top)
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
