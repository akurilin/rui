use crate::pages::layout::VStack;
use crate::pages::{AppPage, Viewport};
use sdl3::pixels::Color;
use sdl3::render::{FRect, WindowCanvas};

pub struct CornersPage;

impl CornersPage {
    pub fn new() -> Self {
        Self
    }
}

impl AppPage for CornersPage {
    fn update(&mut self, _dt_seconds: f32) {}

    fn render(&self, canvas: &mut WindowCanvas, viewport: Viewport) -> Result<(), String> {
        VStack::new(Color::RGB(228, 236, 248))
            .with_spacing(8.0)
            .fill_both()
            .render(canvas, viewport)?;

        let button_w = 128.0;
        let button_h = 40.0;
        let inset = 20.0;
        let max_x = viewport.width as f32 - button_w - inset;
        let max_y = viewport.height as f32 - button_h - inset;
        let mid_x = (viewport.width as f32 - button_w) * 0.5;
        let mid_y = (viewport.height as f32 - button_h) * 0.5;

        let rects = [
            FRect::new(inset, inset, button_w, button_h),
            FRect::new(mid_x, inset, button_w, button_h),
            FRect::new(max_x, inset, button_w, button_h),
            FRect::new(inset, mid_y, button_w, button_h),
            FRect::new(max_x, mid_y, button_w, button_h),
            FRect::new(inset, max_y, button_w, button_h),
            FRect::new(mid_x, max_y, button_w, button_h),
            FRect::new(max_x, max_y, button_w, button_h),
        ];

        canvas.set_draw_color(Color::RGB(61, 78, 109));
        for rect in rects {
            canvas.fill_rect(rect).map_err(|e| e.to_string())?;
        }

        canvas.set_draw_color(Color::RGB(255, 255, 255));
        canvas
            .draw_debug_text("Corners Page (Rust prototype)", (20.0, 18.0))
            .map_err(|e| e.to_string())?;
        canvas
            .draw_debug_text(
                "Resize the window and verify the anchor positions.",
                (20.0, 34.0),
            )
            .map_err(|e| e.to_string())?;

        Ok(())
    }
}
