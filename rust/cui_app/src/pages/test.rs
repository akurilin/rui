use crate::pages::layout::{ColorBlock, HStack, SizeMode, VStack};
use crate::pages::{AppPage, Viewport};
use sdl3::pixels::Color;
use sdl3::render::WindowCanvas;

pub struct TestPage;

impl TestPage {
    pub fn new() -> Self {
        Self
    }
}

impl AppPage for TestPage {
    fn update(&mut self, _dt_seconds: f32) {}

    fn render(&self, canvas: &mut WindowCanvas, viewport: Viewport) -> Result<(), String> {
        let fixed_size_row = HStack::new(Color::RGB(31, 76, 138))
            .with_width(SizeMode::FillParent)
            .with_height(SizeMode::FitContent)
            .with_padding(16.0)
            .with_spacing(16.0)
            .push(ColorBlock::new(Color::RGB(235, 111, 76)).fixed_size(160.0, 96.0))
            .push(ColorBlock::new(Color::RGB(118, 200, 147)).fixed_size(320.0, 144.0))
            .push(ColorBlock::new(Color::RGB(255, 207, 92)).fixed_size(220.0, 72.0));

        let fixed_size_row_fit_width = HStack::new(Color::RGB(41, 92, 162))
            .with_width(SizeMode::FitContent)
            .with_height(SizeMode::FitContent)
            .with_padding(16.0)
            .with_spacing(16.0)
            .push(ColorBlock::new(Color::RGB(235, 111, 76)).fixed_size(160.0, 96.0))
            .push(ColorBlock::new(Color::RGB(118, 200, 147)).fixed_size(320.0, 144.0))
            .push(ColorBlock::new(Color::RGB(255, 207, 92)).fixed_size(220.0, 72.0));

        let fixed_size_row_fit_width_2 = HStack::new(Color::RGB(51, 108, 178))
            .with_width(SizeMode::FitContent)
            .with_height(SizeMode::FitContent)
            .with_padding(16.0)
            .with_spacing(16.0)
            .push(ColorBlock::new(Color::RGB(235, 111, 76)).fixed_size(160.0, 96.0))
            .push(ColorBlock::new(Color::RGB(118, 200, 147)).fixed_size(320.0, 144.0))
            .push(ColorBlock::new(Color::RGB(255, 207, 92)).fixed_size(220.0, 72.0));

        let grow_row = HStack::new(Color::RGB(24, 66, 125))
            .with_width(SizeMode::FillParent)
            .with_height(SizeMode::Grow(1.0))
            .with_padding(8.0)
            .with_spacing(8.0)
            .push(
                ColorBlock::new(Color::RGB(120, 172, 240))
                    .with_width(SizeMode::Grow(1.0))
                    .with_height(SizeMode::FillParent),
            )
            .push(
                ColorBlock::new(Color::RGB(96, 152, 227))
                    .with_width(SizeMode::Grow(2.0))
                    .with_height(SizeMode::FillParent),
            )
            .push(
                ColorBlock::new(Color::RGB(74, 132, 212))
                    .with_width(SizeMode::Grow(1.0))
                    .with_height(SizeMode::FillParent),
            );

        VStack::new(Color::RGB(52, 116, 214))
            .with_spacing(4.0)
            .with_padding(8.0)
            .fill_both()
            .push(fixed_size_row)
            .push(fixed_size_row_fit_width)
            .push(fixed_size_row_fit_width_2)
            .push(grow_row)
            .render(canvas, viewport)
    }
}
