//! This module handles setting up and rendering changes to the UI

use std::borrow::Cow;
use std::cell::RefCell;
use std::rc::Rc;

use native_windows_gui::{self as nwg, ControlHandle, GridLayoutItem, ImageDecoder, WindowFlags};

use windows_sys::Win32::Foundation::COLORREF;
use windows_sys::Win32::Graphics::Gdi::{SetBkColor, SetTextColor};
use windows_sys::Win32::UI::WindowsAndMessaging::WM_CTLCOLORSTATIC;

use crate::resource::{
    BANNER_DESC, CANCEL_BUTTON_TEXT, DOWNLOAD_BUTTON_TEXT, WINDOW_HEIGHT, WINDOW_TITLE,
    WINDOW_WIDTH,
};

use super::delegate::QueueContext;

static BANNER_IMAGE_DATA: &[u8] = include_bytes!("../logo.png");

const BACKGROUND_COLOR: [u8; 3] = [0x19, 0x2e, 0x45];

/// Custom window message handler used to adjust the banner text color.
pub const SET_LABEL_HANDLER_ID: usize = 0x10000;
/// Unique ID of the handler used to handle our custom `QUEUE_MESSAGE`.
pub const QUEUE_MESSAGE_HANDLER_ID: usize = 0x10001;
/// Custom window message used to process requests from other threads.
pub const QUEUE_MESSAGE: u32 = 0x10001;

#[derive(Default)]
pub struct AppWindow {
    pub window: nwg::Window,

    pub grid: nwg::GridLayout,

    pub banner: nwg::ImageFrame,

    pub banner_text: nwg::Label,
    pub banner_image_bitmap: RefCell<Option<nwg::Bitmap>>,

    pub banner_image: nwg::ImageFrame,
    pub cancel_button: nwg::Button,
    pub download_button: nwg::Button,

    pub progress_bar: nwg::ProgressBar,

    pub status_text: nwg::Label,
}

impl AppWindow {
    /// Set up UI elements, position them, and register window message handlers
    /// Note that some additional setup happens in [Self::on_init]
    pub fn layout(mut self) -> Result<Rc<RefCell<AppWindow>>, nwg::NwgError> {
        nwg::Window::builder()
            .size((WINDOW_WIDTH as i32, WINDOW_HEIGHT as i32))
            .center(true)
            .title(WINDOW_TITLE)
            .flags(WindowFlags::WINDOW)
            .build(&mut self.window)?;

        nwg::ImageFrame::builder()
            .parent(&self.window)
            .background_color(Some(BACKGROUND_COLOR))
            .build(&mut self.banner)?;

        nwg::Label::builder()
            .parent(&self.banner)
            .background_color(Some(BACKGROUND_COLOR))
            .build(&mut self.banner_text)?;

        nwg::ImageFrame::builder()
            .parent(&self.banner)
            .background_color(Some(BACKGROUND_COLOR))
            .build(&mut self.banner_image)?;

        nwg::Button::builder()
            .parent(&self.window)
            .size((150, 32))
            .text(&DOWNLOAD_BUTTON_TEXT.replace("&", "&&"))
            .build(&mut self.download_button)?;

        nwg::Button::builder()
            .parent(&self.window)
            .position((0, 200))
            .text(CANCEL_BUTTON_TEXT)
            .build(&mut self.cancel_button)?;

        nwg::Label::builder()
            .parent(&self.window)
            .size((320, 32))
            .h_align(nwg::HTextAlign::Center)
            .build(&mut self.status_text)?;

        const PROGRESS_BAR_MARGIN: i32 = 48;
        nwg::ProgressBar::builder()
            .parent(&self.window)
            .size((WINDOW_WIDTH as i32 - 2 * PROGRESS_BAR_MARGIN, 16))
            .build(&mut self.progress_bar)?;

        nwg::GridLayout::builder()
            .parent(&self.window)
            .margin([0, 0, 0, 0])
            .spacing(0)
            .max_row(Some(5))
            .max_column(Some(1))
            .child_item(GridLayoutItem::new(&self.banner, 0, 0, 1, 2))
            .build(&mut self.grid)?;

        const LOWER_AREA_TOP: i32 = 204;
        const LOWER_AREA_VERT_MARGIN: i32 = 12;
        self.status_text.set_position(
            (self.window.size().0 / 2) as i32 - (self.status_text.size().0 / 2) as i32,
            LOWER_AREA_TOP,
        );
        self.progress_bar.set_position(
            PROGRESS_BAR_MARGIN,
            self.status_text.position().1
                + self.status_text.size().1 as i32
                + LOWER_AREA_VERT_MARGIN,
        );
        self.download_button.set_position(
            (self.window.size().0 / 2) as i32 - (self.download_button.size().0 / 2) as i32,
            self.progress_bar.position().1
                + self.progress_bar.size().1 as i32
                + LOWER_AREA_VERT_MARGIN,
        );
        self.cancel_button.set_position(
            (self.window.size().0 / 2) as i32 - (self.cancel_button.size().0 / 2) as i32,
            self.download_button.position().1,
        );

        self.window.set_visible(true);

        let event_handle = self.window.handle.clone();
        let app = Rc::new(RefCell::new(self));

        handle_init_and_close_messages(event_handle, app.clone());
        handle_queue_message(event_handle, app.clone()).expect("failed to register queue handler");

        Ok(app)
    }

    /// This function is called when the top-level window has been created
    fn on_init(&self) {
        if let Err(err) = self.load_banner_image() {
            eprintln!("load_banner_image failed: {err}");
            // not fatal, so continue
        }

        if let Err(err) = handle_banner_label_colors(&self.banner.handle, SET_LABEL_HANDLER_ID) {
            eprintln!("handle_banner_label_colors failed: {err}");
            // not fatal, so continue
        }

        self.banner_text.set_text(&wrap_text(BANNER_DESC, 76));
        self.banner_text.set_position(24 + 64 + 12, 32 + 12);
        self.banner_text.set_size(
            WINDOW_WIDTH as u32 - self.banner_text.position().0 as u32 - 12,
            64,
        );
    }

    /// This function is called when user clicks the "X"
    fn on_close(&self) {
        nwg::stop_thread_dispatch();
    }

    /// Load the embedded image and display it in `banner_image`
    fn load_banner_image(&self) -> Result<(), nwg::NwgError> {
        const BANNER_SIZE: [u32; 2] = [32, 32];

        let src = ImageDecoder::new()?.from_stream(BANNER_IMAGE_DATA)?;

        let frame = src.frame(0)?;
        let resized_img = ImageDecoder::new()?.resize_image(&frame, BANNER_SIZE)?;

        let b = resized_img.as_bitmap()?;
        let mut img = self.banner_image_bitmap.borrow_mut();
        img.replace(b);

        self.banner_image.set_bitmap(img.as_ref());
        self.banner_image.set_position(24, 32 + 12);
        self.banner_image.set_size(64, 64);

        Ok(())
    }
}

/// Register a window message handler that ensures that the banner labels are rendered with the
/// correct color
fn handle_banner_label_colors(
    banner: &ControlHandle,
    handler_id: usize,
) -> Result<nwg::RawEventHandler, nwg::NwgError> {
    nwg::bind_raw_event_handler(banner, handler_id, move |_hwnd, msg, w, _p| {
        /// This is the RGB() macro except it takes in a slice representing RGB values
        pub fn rgb(color: [u8; 3]) -> COLORREF {
            color[0] as COLORREF | ((color[1] as COLORREF) << 8) | ((color[2] as COLORREF) << 16)
        }

        if msg == WM_CTLCOLORSTATIC {
            unsafe {
                SetTextColor(w as _, rgb([255, 255, 255]));
                SetBkColor(w as _, rgb(BACKGROUND_COLOR));
            }
        }
        None
    })
}

/// Register events for [AppWindow::on_init] and [AppWindow::on_close].
fn handle_init_and_close_messages(
    window: impl Into<ControlHandle>,
    app: Rc<RefCell<AppWindow>>,
) -> nwg::EventHandler {
    let window = window.into();
    nwg::full_bind_event_handler(&window, move |event, _data, handle| match event {
        nwg::Event::OnInit if &handle == &window => {
            let app = app.borrow();
            app.on_init();
        }
        nwg::Event::OnWindowClose if &handle == &window => {
            let app = app.borrow();
            app.on_close();
        }
        _ => (),
    })
}

/// This handles `QUEUE_MESSAGE` messages, which contain callbacks reachable from
/// pointers to a [super::delegate::QueueContext]. See [super::delegate::QueueContext]
/// and [super::delegate::Queue] for details.
fn handle_queue_message(
    window: impl Into<ControlHandle>,
    app: Rc<RefCell<AppWindow>>,
) -> Result<nwg::RawEventHandler, nwg::NwgError> {
    nwg::bind_raw_event_handler(
        &window.into(),
        QUEUE_MESSAGE_HANDLER_ID,
        move |_hwnd, msg, _w, p| {
            if msg == QUEUE_MESSAGE {
                // SAFETY: This message is only sent with a boxed sendable function pointer, so we're
                // good. See the implementation of `AppDelegateQueue` for `Queue`.
                let context = unsafe { Box::from_raw(p as *mut QueueContext) };
                let mut app = app.borrow_mut();
                (context.callback)(&mut *app);
            }
            None
        },
    )
}

/// Insert newlines to wrap text such that the max line length is `cols`
fn wrap_text(s: &str, cols: usize) -> Cow<'_, str> {
    if s.len() <= cols {
        return Cow::Borrowed(s);
    }
    let Some(whitespace) = s[..cols].rfind(' ') else {
        return Cow::Borrowed(s);
    };
    let mut new_str = String::new();
    new_str.push_str(&s[..whitespace]);
    new_str.push('\n');
    if whitespace < s.len() {
        new_str.push_str(&wrap_text(&s[whitespace + 1..], cols));
    }
    Cow::Owned(new_str)
}
