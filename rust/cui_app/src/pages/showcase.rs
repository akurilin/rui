use crate::pages::layout::VStack;
use crate::pages::{AppPage, Viewport};
use sdl3::pixels::Color;
use sdl3::render::{FRect, WindowCanvas};

pub struct ShowcasePage {
    pulse: f32,
}

impl ShowcasePage {
    pub fn new() -> Self {
        Self { pulse: 0.0 }
    }
}

impl AppPage for ShowcasePage {
    fn update(&mut self, dt_seconds: f32) {
        self.pulse += dt_seconds;
    }

    fn render(&self, canvas: &mut WindowCanvas, viewport: Viewport) -> Result<(), String> {
        VStack::new(Color::RGB(245, 238, 230))
            .with_spacing(8.0)
            .fill_both()
            .render(canvas, viewport)?;

        let top = 80.0;
        let left = 40.0;
        let width = viewport.width as f32 - 80.0;
        let card_h = 86.0;

        let animated_blue = ((self.pulse.sin() * 0.5 + 0.5) * 110.0) as u8 + 80;

        canvas.set_draw_color(Color::RGB(75, 48, 35));
        canvas
            .draw_debug_text("Showcase Page (Rust prototype)", (left, 32.0))
            .map_err(|e| e.to_string())?;

        canvas.set_draw_color(Color::RGB(252, 250, 247));
        canvas
            .fill_rect(FRect::new(left, top, width, card_h))
            .map_err(|e| e.to_string())?;
        canvas
            .fill_rect(FRect::new(left, top + 104.0, width, card_h))
            .map_err(|e| e.to_string())?;
        canvas
            .fill_rect(FRect::new(left, top + 208.0, width, card_h))
            .map_err(|e| e.to_string())?;

        canvas.set_draw_color(Color::RGB(29, 89, animated_blue));
        canvas
            .fill_rect(FRect::new(left + 20.0, top + 24.0, 120.0, 36.0))
            .map_err(|e| e.to_string())?;
        canvas.set_draw_color(Color::RGB(46, 112, 70));
        canvas
            .fill_rect(FRect::new(left + 20.0, top + 128.0, 120.0, 36.0))
            .map_err(|e| e.to_string())?;
        canvas.set_draw_color(Color::RGB(168, 75, 35));
        canvas
            .fill_rect(FRect::new(left + 20.0, top + 232.0, 120.0, 36.0))
            .map_err(|e| e.to_string())?;

        canvas.set_draw_color(Color::RGB(50, 45, 41));
        canvas
            .draw_debug_text("Button / Accent sample", (left + 168.0, top + 44.0))
            .map_err(|e| e.to_string())?;
        canvas
            .draw_debug_text("Checkbox + text sample", (left + 168.0, top + 148.0))
            .map_err(|e| e.to_string())?;
        canvas
            .draw_debug_text("Slider + status sample", (left + 168.0, top + 252.0))
            .map_err(|e| e.to_string())?;

        Ok(())
    }
}
