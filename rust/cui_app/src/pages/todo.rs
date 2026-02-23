use crate::pages::layout::VStack;
use crate::pages::{AppPage, Viewport};
use sdl3::pixels::Color;
use sdl3::render::{FRect, WindowCanvas};

pub struct TodoPage {
    elapsed_seconds: f32,
}

impl TodoPage {
    pub fn new() -> Self {
        Self {
            elapsed_seconds: 0.0,
        }
    }
}

impl AppPage for TodoPage {
    fn update(&mut self, dt_seconds: f32) {
        self.elapsed_seconds += dt_seconds;
    }

    fn render(&self, canvas: &mut WindowCanvas, viewport: Viewport) -> Result<(), String> {
        VStack::new(Color::RGB(241, 241, 238))
            .with_spacing(8.0)
            .fill_both()
            .render(canvas, viewport)?;

        canvas.set_draw_color(Color::RGB(31, 58, 80));
        canvas
            .fill_rect(FRect::new(36.0, 56.0, viewport.width as f32 - 72.0, 64.0))
            .map_err(|e| e.to_string())?;

        canvas.set_draw_color(Color::RGB(255, 255, 255));
        canvas
            .draw_debug_text("TODO (Rust prototype)", (56.0, 84.0))
            .map_err(|e| e.to_string())?;

        canvas.set_draw_color(Color::RGB(72, 72, 72));
        canvas
            .fill_rect(FRect::new(36.0, 140.0, viewport.width as f32 - 72.0, 56.0))
            .map_err(|e| e.to_string())?;
        canvas.set_draw_color(Color::RGB(240, 240, 240));
        canvas
            .draw_debug_text("Add a task...", (56.0, 172.0))
            .map_err(|e| e.to_string())?;

        canvas.set_draw_color(Color::RGB(252, 252, 252));
        canvas
            .fill_rect(FRect::new(
                36.0,
                220.0,
                viewport.width as f32 - 72.0,
                viewport.height as f32 - 270.0,
            ))
            .map_err(|e| e.to_string())?;

        canvas.set_draw_color(Color::RGB(56, 56, 56));
        canvas
            .draw_debug_text("[ ] Review Rust port structure", (56.0, 260.0))
            .map_err(|e| e.to_string())?;
        canvas
            .draw_debug_text("[ ] Port runtime/event routing", (56.0, 292.0))
            .map_err(|e| e.to_string())?;
        canvas
            .draw_debug_text("[ ] Port layout container + scroll view", (56.0, 324.0))
            .map_err(|e| e.to_string())?;
        canvas
            .draw_debug_text("[ ] Port full TODO interactions", (56.0, 356.0))
            .map_err(|e| e.to_string())?;
        canvas
            .draw_debug_text(
                &format!("prototype uptime: {:.1}s", self.elapsed_seconds),
                (56.0, viewport.height as f32 - 40.0),
            )
            .map_err(|e| e.to_string())?;

        Ok(())
    }
}
