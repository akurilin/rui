use crate::pages::layout::{ColorBlock, HStack, SizeMode, TextLabel, VStack};
use crate::pages::{AppPage, Viewport};
use sdl3::pixels::Color;
use sdl3::render::WindowCanvas;

const ROOT_STACK_PADDING: f32 = 20.0;
const ROOT_STACK_SPACING: f32 = 3.0;
const ROW_FIXED_PADDING: f32 = 12.0;
const ROW_FIXED_SPACING: f32 = 16.0;

pub struct TestPage;

impl TestPage {
    pub fn new() -> Self {
        Self
    }
}

impl AppPage for TestPage {
    fn update(&mut self, _dt_seconds: f32) {}

    fn render(&self, canvas: &mut WindowCanvas, viewport: Viewport) -> Result<(), String> {
        let fixed_block_color = Color::RGB(235, 111, 76);
        let grow_block_color = Color::RGB(118, 200, 147);

        // Row 1: HStack fills parent width, children are fixed-size blocks.
        let fixed_size_row = HStack::new(Color::RGB(31, 76, 138))
            .with_width(SizeMode::FillParent)
            .with_height(SizeMode::FitContent)
            .with_padding(ROW_FIXED_PADDING)
            .with_spacing(ROW_FIXED_SPACING)
            .push(with_layout_label(
                ColorBlock::new(fixed_block_color).fixed_size(160.0, 72.0),
                "fixed/fixed",
            ))
            .push(with_layout_label(
                ColorBlock::new(fixed_block_color).fixed_size(320.0, 108.0),
                "fixed/fixed",
            ))
            .push(with_layout_label(
                ColorBlock::new(fixed_block_color).fixed_size(220.0, 27.0),
                "fixed/fixed",
            ));

        // Row 2: HStack fit-content width with only fixed-size children (shrink-wrap behavior).
        let fixed_size_row_fit_width = HStack::new(Color::RGB(41, 92, 162))
            .with_width(SizeMode::FitContent)
            .with_height(SizeMode::FitContent)
            .with_padding(ROW_FIXED_PADDING)
            .with_spacing(ROW_FIXED_SPACING)
            .push(with_layout_label(
                ColorBlock::new(fixed_block_color).fixed_size(160.0, 72.0),
                "fixed/fixed",
            ))
            .push(with_layout_label(
                ColorBlock::new(fixed_block_color).fixed_size(320.0, 108.0),
                "fixed/fixed",
            ))
            .push(with_layout_label(
                ColorBlock::new(fixed_block_color).fixed_size(220.0, 27.0),
                "fixed/fixed",
            ));

        // Row 3: HStack fit-content width with a grow child on the main axis.
        // Current policy promotes fit-content to available width when any main-axis grow child exists.
        let fixed_size_row_fit_width_2 = HStack::new(Color::RGB(51, 108, 178))
            .with_width(SizeMode::FitContent)
            .with_height(SizeMode::FitContent)
            .with_padding(ROW_FIXED_PADDING)
            .with_spacing(ROW_FIXED_SPACING)
            .push(with_layout_label(
                ColorBlock::new(fixed_block_color).fixed_size(160.0, 72.0),
                "fixed/fixed",
            ))
            .push(with_layout_label(
                ColorBlock::new(grow_block_color).size(SizeMode::Grow(1.0), SizeMode::Fixed(108.0)),
                "grow(1)/fixed",
            ))
            .push(with_layout_label(
                ColorBlock::new(fixed_block_color).fixed_size(220.0, 27.0),
                "fixed/fixed",
            ));

        // Row 4: HStack fill-parent width with a grow middle child consuming remaining width.
        let fixed_size_row_fill_width_3 = HStack::new(Color::RGB(51, 108, 178))
            .with_width(SizeMode::FillParent)
            .with_height(SizeMode::FitContent)
            .with_padding(ROW_FIXED_PADDING)
            .with_spacing(ROW_FIXED_SPACING)
            .push(with_layout_label(
                ColorBlock::new(fixed_block_color).fixed_size(160.0, 72.0),
                "fixed/fixed",
            ))
            .push(with_layout_label(
                ColorBlock::new(grow_block_color).size(SizeMode::Grow(1.0), SizeMode::Fixed(108.0)),
                "grow(1)/fixed",
            ))
            .push(with_layout_label(
                ColorBlock::new(fixed_block_color).fixed_size(220.0, 27.0),
                "fixed/fixed",
            ));

        // Row 5: HStack fit-content width with two grow children on the main axis.
        // Current policy promotes fit-content to available width when any main-axis grow child exists.
        let fixed_size_row_fit_width_4 = HStack::new(Color::RGB(51, 108, 178))
            .with_width(SizeMode::FitContent)
            .with_height(SizeMode::FitContent)
            .with_padding(ROW_FIXED_PADDING)
            .with_spacing(ROW_FIXED_SPACING)
            .push(with_layout_label(
                ColorBlock::new(fixed_block_color).fixed_size(160.0, 72.0),
                "fixed/fixed",
            ))
            .push(with_layout_label(
                ColorBlock::new(grow_block_color).size(SizeMode::Grow(1.0), SizeMode::Fixed(108.0)),
                "grow(1)/fixed",
            ))
            .push(with_layout_label(
                ColorBlock::new(grow_block_color).size(SizeMode::Grow(2.0), SizeMode::Fixed(27.0)),
                "grow(2)/fixed",
            ));

        // Row 6: HStack grows vertically within the VStack; all children split width by grow weight.
        let grow_row = HStack::new(Color::RGB(24, 66, 125))
            .with_width(SizeMode::FillParent)
            .with_height(SizeMode::Grow(1.0))
            .with_padding(6.0)
            .with_spacing(8.0)
            .push(with_layout_label(
                ColorBlock::new(grow_block_color)
                    .with_width(SizeMode::Grow(1.0))
                    .with_height(SizeMode::FillParent),
                "grow(1)/fill",
            ))
            .push(with_layout_label(
                ColorBlock::new(grow_block_color)
                    .with_width(SizeMode::Grow(2.0))
                    .with_height(SizeMode::FillParent),
                "grow(2)/fill",
            ))
            .push(with_layout_label(
                ColorBlock::new(grow_block_color)
                    .with_width(SizeMode::Grow(1.0))
                    .with_height(SizeMode::FillParent),
                "grow(1)/fill",
            ));

        // The actual container of all of the rows.
        VStack::new(Color::RGB(52, 116, 214))
            .with_spacing(ROOT_STACK_SPACING)
            .with_padding(ROOT_STACK_PADDING)
            .fill_both()
            .push(fixed_size_row)
            .push(fixed_size_row_fit_width)
            .push(fixed_size_row_fit_width_2)
            .push(fixed_size_row_fill_width_3)
            .push(fixed_size_row_fit_width_4)
            .push(grow_row)
            .render(canvas, viewport)
    }
}

fn with_layout_label(block: ColorBlock, label: &str) -> ColorBlock {
    block.push_overlay_center(TextLabel::new(label).with_color(Color::RGB(19, 24, 37)))
}
