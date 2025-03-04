//! This module handles setting up and rendering changes to the UI

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::LazyLock;

use native_windows_gui::{self as nwg, ControlHandle, ImageDecoder, WindowFlags};

use windows_sys::Win32::Foundation::COLORREF;
use windows_sys::Win32::Graphics::Gdi::{
    CreateFontIndirectW, SetBkColor, SetBkMode, SetTextColor, COLOR_WINDOW, LOGFONTW, TRANSPARENT,
};
use windows_sys::Win32::UI::WindowsAndMessaging::WM_CTLCOLORSTATIC;

use crate::resource::{
    BANNER_DESC, BETA_LINK_TEXT, BETA_PREFACE_DESC, CANCEL_BUTTON_SIZE, CANCEL_BUTTON_TEXT,
    DOWNLOAD_BUTTON_SIZE, DOWNLOAD_BUTTON_TEXT, STABLE_LINK_TEXT, WINDOW_HEIGHT, WINDOW_TITLE,
    WINDOW_WIDTH,
};

use super::delegate::QueueContext;

static BANNER_IMAGE_DATA: &[u8] = include_bytes!("../../assets/logo-icon.png");
static BANNER_TEXT_IMAGE_DATA: &[u8] = include_bytes!("../../assets/logo-text.png");
static ERROR_IMAGE_DATA: &[u8] = include_bytes!("../../assets/alert-circle.png");

const BACKGROUND_COLOR: [u8; 3] = [0x19, 0x2e, 0x45];
/// Beta link color: #003E92
const LINK_COLOR: [u8; 3] = [0x00, 0x3e, 0x92];

/// Custom window message handler used to adjust the banner text color.
pub const SET_LABEL_HANDLER_ID: usize = 0x10000;
/// Unique ID of the handler used to handle our custom `QUEUE_MESSAGE`.
pub const QUEUE_MESSAGE_HANDLER_ID: usize = 0x10001;
/// Custom window message used to process requests from other threads.
pub const QUEUE_MESSAGE: u32 = 0x10001;
/// Unique ID of the handler for the stable link prefix.
pub const STABLE_LINK_PREFIX_HANDLER_ID: usize = 0x10004;
/// Unique ID of the handler for the stable link.
pub const STABLE_LINK_HANDLER_ID: usize = 0x10003;
/// Unique ID of the handler for the beta link.
pub const BETA_LINK_HANDLER_ID: usize = 0x10002;

#[derive(Default)]
pub struct AppWindow {
    pub window: nwg::Window,

    pub banner: nwg::ImageFrame,

    pub banner_text: nwg::Label,
    pub banner_text_image_bitmap: RefCell<Option<nwg::Bitmap>>,
    pub banner_text_image: nwg::ImageFrame,
    pub banner_image_bitmap: RefCell<Option<nwg::Bitmap>>,
    pub banner_image: nwg::ImageFrame,

    pub cancel_button: nwg::Button,
    pub download_button: nwg::Button,

    pub progress_bar: nwg::ProgressBar,

    pub status_text: nwg::Label,
    pub download_text: nwg::Label,

    pub beta_prefix: nwg::Label,
    pub beta_link: nwg::Label,

    pub arrow_font: nwg::Font,

    pub stable_message_frame: nwg::ImageFrame,
    pub stable_prefix: nwg::Label,
    pub stable_link: nwg::Label,

    pub error_view: ErrorView,
}

#[derive(Default)]
pub struct ErrorView {
    pub error_frame: nwg::Frame,
    pub error_text: nwg::Label,
    pub error_icon: nwg::ImageFrame,
    pub error_icon_bmp: nwg::Bitmap,
    pub error_cancel_button: nwg::Button,
    pub error_retry_button: nwg::Button,
}

impl ErrorView {
    pub fn layout(&mut self, parent: &nwg::ControlHandle) -> Result<(), nwg::NwgError> {
        nwg::Frame::builder()
            .parent(parent)
            .position((0, 102))
            .size((WINDOW_WIDTH as i32, 204))
            .flags(nwg::FrameFlags::empty())
            .build(&mut self.error_frame)?;

        nwg::Label::builder()
            .parent(&self.error_frame)
            .v_align(nwg::VTextAlign::Center)
            .position((80, 45))
            .size((488, 64))
            .build(&mut self.error_text)?;

        nwg::ImageFrame::builder()
            .parent(&self.error_frame)
            .size((32, 32))
            .position((34, 49))
            .build(&mut self.error_icon)?;

        let button_y =
            self.error_text.position().1 + i32::try_from(self.error_text.size().1).unwrap() + 11;

        nwg::Button::builder()
            .parent(&self.error_frame)
            .position((304, button_y))
            .size((232, 32))
            .build(&mut self.error_cancel_button)?;

        nwg::Button::builder()
            .parent(&self.error_frame)
            .position((64, button_y))
            .size((232, 32))
            .build(&mut self.error_retry_button)?;

        self.load_error_icon()?;

        Ok(())
    }

    /// Load the error icon and display it in `error_icon`
    fn load_error_icon(&mut self) -> Result<(), nwg::NwgError> {
        let src = ImageDecoder::new()?.from_stream(ERROR_IMAGE_DATA)?;
        let frame = src.frame(0)?;
        self.error_icon_bmp = frame.as_bitmap().unwrap();
        self.error_icon.set_bitmap(Some(&self.error_icon_bmp));
        Ok(())
    }
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
        nwg::ImageFrame::builder()
            .parent(&self.banner)
            .background_color(Some(BACKGROUND_COLOR))
            .build(&mut self.banner_text_image)?;

        nwg::Button::builder()
            .parent(&self.window)
            .size(try_pair_into(DOWNLOAD_BUTTON_SIZE).unwrap())
            .text(&DOWNLOAD_BUTTON_TEXT.replace("&", "&&"))
            .build(&mut self.download_button)?;

        nwg::Button::builder()
            .parent(&self.window)
            .size(try_pair_into(CANCEL_BUTTON_SIZE).unwrap())
            .text(CANCEL_BUTTON_TEXT)
            .build(&mut self.cancel_button)?;

        nwg::Label::builder()
            .parent(&self.window)
            .size((320, 32))
            .text("")
            .h_align(nwg::HTextAlign::Center)
            .build(&mut self.status_text)?;

        nwg::Label::builder()
            .parent(&self.window)
            .size((480, 32))
            .text("")
            .h_align(nwg::HTextAlign::Center)
            .build(&mut self.download_text)?;

        nwg::Label::builder()
            .parent(&self.window)
            .size((240, 24))
            .text(BETA_PREFACE_DESC)
            .h_align(nwg::HTextAlign::Left)
            .build(&mut self.beta_prefix)?;

        let link_font = create_link_font()?;

        nwg::Label::builder()
            .parent(&self.window)
            .size((128, 24))
            .text(BETA_LINK_TEXT)
            .font(Some(link_font))
            .h_align(nwg::HTextAlign::Left)
            .build(&mut self.beta_link)?;

        nwg::ImageFrame::builder()
            .parent(&self.window)
            .size((240, 24))
            .build(&mut self.stable_message_frame)?;

        nwg::Font::builder()
            // TODO: Ensure font always exists
            .family("Segoe Fluent Icons")
            .size(10)
            .build(&mut self.arrow_font)?;
        nwg::Label::builder()
            .parent(&self.stable_message_frame)
            .size((16, 24))
            .text("")
            .font(Some(&self.arrow_font))
            .h_align(nwg::HTextAlign::Left)
            .build(&mut self.stable_prefix)?;
        nwg::Label::builder()
            .parent(&self.stable_message_frame)
            .size((240, 24))
            .text(STABLE_LINK_TEXT)
            .font(Some(link_font))
            .h_align(nwg::HTextAlign::Left)
            .build(&mut self.stable_link)?;

        const PROGRESS_BAR_MARGIN: i32 = 48;
        nwg::ProgressBar::builder()
            .parent(&self.window)
            .size((WINDOW_WIDTH as i32 - 2 * PROGRESS_BAR_MARGIN, 16))
            .build(&mut self.progress_bar)?;

        const BANNER_HEIGHT: u32 = 102;

        self.banner.set_size(self.window.size().0, BANNER_HEIGHT);

        const LOWER_AREA_YMARGIN: i32 = 48;
        const LOWER_AREA_YPADDING: i32 = 16;
        const LABEL_YSPACING: i32 = 16;

        self.download_text.set_visible(false);
        self.status_text.set_position(
            (self.window.size().0 / 2) as i32 - (self.status_text.size().0 / 2) as i32,
            BANNER_HEIGHT as i32 + LOWER_AREA_YMARGIN,
        );
        self.download_button.set_position(
            (self.window.size().0 / 2) as i32 - (self.download_button.size().0 / 2) as i32,
            self.status_text.position().1 + 8 + LABEL_YSPACING + LOWER_AREA_YPADDING,
        );
        self.download_text.set_position(
            (self.window.size().0 / 2) as i32 - (self.download_text.size().0 / 2) as i32,
            self.status_text.position().1 + LABEL_YSPACING + LOWER_AREA_YPADDING,
        );
        self.progress_bar.set_position(
            PROGRESS_BAR_MARGIN,
            self.download_text.position().1 + LABEL_YSPACING + LOWER_AREA_YPADDING,
        );
        self.cancel_button.set_position(
            (self.window.size().0 / 2) as i32 - (self.cancel_button.size().0 / 2) as i32,
            self.progress_bar.position().1
                + self.progress_bar.size().1 as i32
                + LOWER_AREA_YPADDING,
        );

        self.stable_message_frame.set_position(
            24,
            self.window.size().1 as i32 - 24 - self.stable_message_frame.size().1 as i32,
        );
        self.stable_link.set_position(16, 0);
        self.stable_prefix.set_position(4, 12 - 4);
        handle_link_messages(
            &self.stable_message_frame.handle,
            &self.stable_prefix,
            STABLE_LINK_PREFIX_HANDLER_ID,
        )?;
        handle_link_messages(
            &self.stable_message_frame.handle,
            &self.stable_link,
            STABLE_LINK_HANDLER_ID,
        )?;

        self.beta_prefix.set_position(
            24,
            self.window.size().1 as i32 - 24 - self.beta_prefix.size().1 as i32,
        );
        self.beta_link.set_position(
            self.beta_prefix.position().0 + self.beta_prefix.size().0 as i32,
            self.beta_prefix.position().1,
        );
        handle_link_messages(&self.window.handle, &self.beta_link, BETA_LINK_HANDLER_ID)?;

        self.window.set_visible(true);

        self.error_view.layout(&self.window.handle)?;

        let event_handle = self.window.handle;
        let app = Rc::new(RefCell::new(self));

        handle_init_and_close_messages(event_handle, app.clone());
        handle_queue_message(event_handle, app.clone())?;

        Ok(app)
    }

    /// This function is called when the top-level window has been created
    fn on_init(&self) {
        if let Err(err) = self.load_banner_image() {
            log::error!("load_banner_image failed: {err}");
            // not fatal, so continue
        }
        if let Err(err) = self.load_banner_text_image() {
            log::error!("load_banner_text_image failed: {err}");
            // not fatal, so continue
        }

        if let Err(err) = handle_banner_label_colors(&self.banner.handle, SET_LABEL_HANDLER_ID) {
            log::error!("handle_banner_label_colors failed: {err}");
            // not fatal, so continue
        }

        self.banner_text.set_text(BANNER_DESC);
        self.banner_text
            .set_position(24, self.banner_image.position().1 + 20);
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
        let src = ImageDecoder::new()?.from_stream(BANNER_IMAGE_DATA)?;
        let frame = src.frame(0)?;
        let size = frame.size();
        let mut img = self.banner_image_bitmap.borrow_mut();
        let bmp = frame.as_bitmap()?;
        img.replace(bmp);

        self.banner_image.set_bitmap(img.as_ref());
        self.banner_image.set_position(24, 24);
        self.banner_image.set_size(size.0, size.1);

        Ok(())
    }

    /// Load the embedded image and display it in `banner_text_image`
    fn load_banner_text_image(&self) -> Result<(), nwg::NwgError> {
        let src = ImageDecoder::new()?.from_stream(BANNER_TEXT_IMAGE_DATA)?;
        let frame = src.frame(0)?;
        let size = frame.size();
        let mut img = self.banner_text_image_bitmap.borrow_mut();
        img.replace(frame.as_bitmap()?);

        self.banner_text_image.set_bitmap(img.as_ref());
        self.banner_text_image.set_position(
            self.banner_image.position().0 + self.banner_image.size().0 as i32 + 8,
            self.banner_image.position().1 + self.banner_image.size().1 as i32 / 2
                - size.1 as i32 / 2,
        );
        self.banner_text_image.set_size(size.0, size.1);

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
            // SAFETY: `w` is a valid device context for WM_CTLCOLORSTATIC
            unsafe {
                SetTextColor(w as _, rgb([255, 255, 255]));
                SetBkColor(w as _, rgb(BACKGROUND_COLOR));
            }
        }
        None
    })
}

/// Register a window message handler for the beta link component
fn handle_link_messages(
    parent: &nwg::ControlHandle,
    link: &nwg::Label,
    handler_id: usize,
) -> Result<nwg::RawEventHandler, nwg::NwgError> {
    let link_hwnd = link.handle.hwnd().map(|hwnd| hwnd as isize);
    nwg::bind_raw_event_handler(parent, handler_id, move |_hwnd, msg, w, p| {
        /// This is the RGB() macro except it takes in a slice representing RGB values
        pub fn rgb(color: [u8; 3]) -> COLORREF {
            color[0] as COLORREF | ((color[1] as COLORREF) << 8) | ((color[2] as COLORREF) << 16)
        }

        if msg == WM_CTLCOLORSTATIC && Some(p) == link_hwnd {
            // SAFETY: `w` is a valid device context for WM_CTLCOLORSTATIC
            unsafe {
                SetBkMode(w as _, TRANSPARENT as _);
                SetTextColor(w as _, rgb(LINK_COLOR));
            }
            // Out of bounds background
            return Some(COLOR_WINDOW as _);
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
        nwg::Event::OnInit if handle == window => {
            let app = app.borrow();
            app.on_init();
        }
        nwg::Event::OnWindowClose if handle == window => {
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
                (context.callback)(&mut app);
            }
            None
        },
    )
}

fn try_pair_into<A: TryInto<B>, B>(a: (A, A)) -> Result<(B, B), A::Error> {
    Ok((a.0.try_into()?, a.1.try_into()?))
}

/// Create a link font
///
/// NOTE: The font is never freed using DeleteObject. This is acceptable since it exists for the
///       lifetime of the program.
fn create_link_font() -> Result<&'static nwg::Font, nwg::NwgError> {
    static LINK_FONT: LazyLock<Result<nwg::Font, nwg::NwgError>> = LazyLock::new(|| {
        let face_name = "Segoe UI".encode_utf16();

        // SAFETY: Trivially safe. `LOGFONTW` is a C struct
        let mut logfont: LOGFONTW = unsafe { std::mem::zeroed() };
        logfont.lfUnderline = 1;
        for (dest, src) in logfont.lfFaceName.iter_mut().zip(face_name) {
            *dest = src;
        }

        // SAFETY: `logfont` is a valid font
        let raw_font = unsafe { CreateFontIndirectW(&logfont) };

        if raw_font == 0 {
            return Err(nwg::NwgError::Unknown);
        }

        Ok(nwg::Font {
            handle: raw_font as _,
        })
    });

    match &*LINK_FONT {
        Ok(font) => Ok(font),
        Err(err) => Err(err.to_owned()),
    }
}
