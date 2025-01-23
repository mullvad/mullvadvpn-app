use std::borrow::{Borrow, BorrowMut};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, LazyLock, Mutex};

use cacao::button::Button;
use cacao::color::Color;
use cacao::image::{Image, ImageView};
use cacao::layout::{Layout, LayoutConstraint};
use cacao::notification_center::Dispatcher;
use cacao::progress::ProgressIndicator;
use cacao::text::{AttributedString, Label};
use cacao::view::View;

use cacao::appkit::window::{Window, WindowConfig, WindowDelegate};
use cacao::appkit::{App, AppDelegate};

const WINDOW_TITLE: &str = "Mullvad VPN installer";
const WINDOW_WIDTH: usize = 676;
const WINDOW_HEIGHT: usize = 390;

/// Logo render in the banner
const LOGO_IMAGE_DATA: &[u8] = include_bytes!("../logo-icon.svg");

/// Logo banner text
const LOGO_TEXT_DATA: &[u8] = include_bytes!("../logo-text.svg");

/// Banner background color
static BANNER_COLOR: LazyLock<Color> = LazyLock::new(|| Color::rgba(0x19, 0x2e, 0x45, 0xff));

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
    DownloadClick(Arc<Mutex<Box<dyn for<'a> Fn(&'a mut AppWindow) + Send>>>),
    /// Run callback on main thread
    QueueMain(Mutex<Option<Box<dyn for<'a> FnOnce(&'a mut AppWindow) + Send>>>),
}

impl Dispatcher for AppImpl {
    type Message = Action;

    fn on_ui_message(&self, message: Self::Message) {
        match message {
            Action::DownloadClick(cb) => {
                let cb = cb.lock().unwrap();
                let delegate = self.window.delegate.as_ref().unwrap();
                // NOTE: We assume that this won't panic because this will never run simultaneously
                //       with `queue_main``
                let mut borrowed = delegate.inner.borrow_mut();
                cb(&mut borrowed);
            }
            Action::QueueMain(cb) => {
                let cb = cb.lock().unwrap().take().unwrap();
                let delegate = self.window.delegate.as_ref().unwrap();
                let mut borrowed = delegate.inner.borrow_mut();
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

    pub beta_link_preface: Label,
    pub beta_link: Label,
}

pub struct DownloadButton {
    pub button: Button,
}

impl Default for DownloadButton {
    fn default() -> Self {
        let button = Button::new("Download & install");
        Self { button }
    }
}

impl AppWindow {
    pub fn layout(&self) {
        self.banner_logo_view.set_image(&LOGO);
        self.banner_logo_text_view.set_image(&LOGO_TEXT);

        //let cg_color = BANNER_COLOR.cg_color();
        //cacao::core_graphics::color_space::kCGColorSpaceGenericRGB
        //let ptr = unsafe { cacao::objc_id::Id::from_ptr(msg_send![class!(NSColor), colorWithCalibratedRed:r green:g blue:b alpha:a]) };

        self.banner.set_background_color(&*BANNER_COLOR);

        // FIXME: doesn't segfault without color space stuff
        /*
        let col = BANNER_COLOR.cg_color();
        let col = CGColor::rgb(
            0x19 as f64 / 255.,
            0x2e as f64 / 255.,
            0x45 as f64 / 255.,
            0xff as f64 / 255.,
        );

        let space = CGColorSpace::create_device_rgb();
        */
        //let new_col: CGColorRef = unsafe { msg_send![col, colorUsingColorSpace: space] };

        // FIXME: segfaults less often without this, but it still happens
        /*self.banner.layer.objc.with_mut(move |obj| unsafe {
            let _: () = msg_send![&*obj, setBackgroundColor: col.clone()];
        });*/

        self.banner.add_subview(&self.banner_logo_view);
        self.banner.add_subview(&self.banner_logo_text_view);

        self.content.add_subview(&self.banner);
        self.content.add_subview(&self.main_view);

        self.main_view.add_subview(&self.progress);
        self.progress.set_hidden(true);
        self.progress.set_indeterminate(false);

        self.banner_desc.set_text("The Mullvad VPN app will be downloaded and then verified to ensure that it is a version that comes from us.");
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

        self.text.set_text("Latest version: 2025.3");
        self.main_view.add_subview(&self.text);
        self.main_view.add_subview(&self.button.button);

        self.beta_link_preface
            .set_text("Want to try out new features? ");
        self.main_view.add_subview(&self.beta_link_preface);

        let attr_text_s = "Try the beta version!";
        let mut attr_text = AttributedString::new(&attr_text_s);
        attr_text.set_text_color(Color::Link, 0..attr_text_s.len() as isize);

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
            self.progress.height.constraint_equal_to_constant(48.0f64),
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
