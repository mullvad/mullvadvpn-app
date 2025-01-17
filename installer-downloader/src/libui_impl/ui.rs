//! UI for app downloader
use std::rc::Rc;

use libui::controls::*;
use libui::draw::{Brush, DrawContext, FillMode, Path, SolidBrush};
use libui::prelude::*;

/// Window title
const WINDOW_TITLE: &str = "Mullvad VPN downloader";

/// Window width
const WINDOW_WIDTH: i32 = 676;

/// Window height
const WINDOW_HEIGHT: i32 = 364;

/// Background color of the top banner
/// #192E45
const TOP_BACKGROUND: Brush = Brush::Solid(SolidBrush {
    r: (0x19 as f64) / 255.0,
    g: (0x2e as f64) / 255.0,
    b: (0x45 as f64) / 255.0,
    a: 1.,
});

/// Logo to render in the top banner
const LOGO_PNG: &[u8] = include_bytes!("../logo.png");

/// Success icon
//const SUCCESS_CHECK: &[u8] = include_bytes!("../logo.png");

/// See the [module-level documentation](self).
#[derive(Clone)]
pub struct AppUi {
    /// Main window
    pub window: Window,
    /// Button that initiates stable download
    pub download_button: Button,
    /// Progress bar to display when downloading the stable version
    pub progress_bar: ProgressBar,
    /// Information displayed under stable download bar
    pub download_text: Label,
}

impl AppUi {
    pub fn new(ui: &UI) -> Self {
        let mut window = Window::new(
            &ui,
            WINDOW_TITLE,
            WINDOW_WIDTH,
            WINDOW_HEIGHT,
            WindowType::NoMenubar,
        );
        window.set_margined(false);
        //win.set_resizeable(false);

        //let checkmark = image::load_from_memory_with_format(SUCCESS_IMG, image::ImageFormat::Png)
        //    .map(|logo| Rc::new(logo.into_rgba32f()))
        //    .ok();
        //let new_success_icon = || SuccessIcon { image: checkmark };

        let mut outer_grid = LayoutGrid::new();
        outer_grid.set_padded(true);

        let logo = image::load_from_memory_with_format(LOGO_PNG, image::ImageFormat::Png)
            .map(|logo| logo.into_rgba32f())
            .ok();

        let area = Area::new(Box::new(LogoAreaHandler { logo }));
        //outer_grid.append(area, LayoutStrategy::Stretchy);
        outer_grid.append(
            area,
            0,
            0,
            1,
            1,
            GridExpand::Both,
            GridAlignment::Fill,
            GridAlignment::Fill,
        );

        // TODO: try macro?

        let mut grid = LayoutGrid::new();
        grid.set_padded(true);

        // Stable row
        let version_label = Label::new("");
        grid.append(
            version_label,
            1,
            0,
            1,
            1,
            GridExpand::Horizontal,
            GridAlignment::Center,
            GridAlignment::Start,
        );
        let download_button = Button::new("Download && install");
        grid.append(
            download_button.clone(),
            1,
            1,
            1,
            1,
            GridExpand::Horizontal,
            GridAlignment::Center,
            GridAlignment::Start,
        );

        let mut progress_bar = ProgressBar::new();
        grid.append(
            progress_bar.clone(),
            1,
            2,
            1,
            1,
            GridExpand::Both,
            GridAlignment::Fill,
            GridAlignment::Start,
        );

        // Progress info text
        let download_text = Label::new("");
        grid.append(
            download_text.clone(),
            1,
            3,
            1,
            1,
            GridExpand::Both,
            GridAlignment::Center,
            GridAlignment::Start,
        );

        outer_grid.append(
            grid,
            0,
            1,
            1,
            2,
            GridExpand::Both,
            GridAlignment::Fill,
            GridAlignment::Fill,
        );
        window.set_child(outer_grid);

        Self {
            window,
            download_button,
            progress_bar,
            download_text,
        }
    }
}

struct SuccessIcon {
    image: Option<Rc<image::Rgba32FImage>>,
}

impl AreaHandler for SuccessIcon {
    fn draw(&mut self, _area: &Area, dp: &AreaDrawParams) {
        let Some(image) = self.image.as_ref() else {
            // No image to render
            return;
        };
        render_image(&dp.context, image);
    }
}

impl SuccessIcon {
    fn into_area(self) -> Area {
        if let Some(image) = self.image.as_ref() {
            let width = image.width();
            let height = image.height();
            println!("width, height: {width}, {height}");
            Area::new_scrolling(Box::new(self), width as i64, height as i64)
        } else {
            Area::new(Box::new(self))
        }
    }
}

struct LogoAreaHandler {
    logo: Option<image::Rgba32FImage>,
}

impl AreaHandler for LogoAreaHandler {
    fn draw(&mut self, _area: &Area, dp: &AreaDrawParams) {
        let ctx = &dp.context;

        // Fill background
        let path = Path::new(ctx, FillMode::Winding);
        path.add_rectangle(ctx, 0., 0., dp.area_width, dp.area_height);
        path.end(ctx);

        dp.context.fill(&path, &TOP_BACKGROUND);

        let Some(logo) = self.logo.as_ref() else {
            // No logo to render
            return;
        };
        render_image(ctx, logo);
    }
}

/// Render image on an `Area`
fn render_image(ctx: &DrawContext, image: &image::Rgba32FImage) {
    for (i, px) in image.pixels().enumerate() {
        let brush = Brush::Solid(SolidBrush {
            r: px[0] as f64,
            g: px[1] as f64,
            b: px[2] as f64,
            a: px[3] as f64,
        });

        let x = i % (image.width() as usize);
        let y = i / (image.width() as usize);

        let pixel_path = Path::new(ctx, FillMode::Winding);
        pixel_path.add_rectangle(ctx, x as f64, y as f64, 1.0, 1.0);
        pixel_path.end(ctx);
        ctx.fill(&pixel_path, &brush);
    }
}
