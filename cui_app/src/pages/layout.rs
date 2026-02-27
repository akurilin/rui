//! Stack-based layout primitives used by the Rust prototype pages.
//!
//! The model is intentionally small:
//! - `VStack` and `HStack` are axis-specialized wrappers over a single `Stack` implementation.
//! - Nodes expose a `SizeMode` per axis (`FitContent`, `FillParent`, `Fixed`, `Grow`).
//! - Layout follows a measure/arrange split:
//!   - Measure computes intrinsic/base size.
//!   - Arrange places children into the final rectangle and distributes remaining space.
//!
//! Important sizing semantics:
//! - `FillParent` and `Grow` are treated like fit-content during measurement.
//! - `Grow(weight)` consumes remaining main-axis space only during arrangement.
//! - Child main-axis sizes are clamped to remaining parent space during arrangement.
//! - Overlay children are parent-relative and never participate in flow sizing/math.

use crate::pages::Viewport;
use sdl3::image::ImageIOStream;
use sdl3::iostream::IOStream;
use sdl3::pixels::Color;
use sdl3::render::{FRect, WindowCanvas};
use sdl3::ttf;

/// One-axis sizing behavior for layout nodes.
#[derive(Clone, Copy)]
pub enum SizeMode {
    /// Use intrinsic/measured size.
    FitContent,
    /// Fill available parent size at arrange-time.
    FillParent,
    /// Use a fixed value in pixels.
    Fixed(f32),
    /// Take a weighted share of leftover space at arrange-time.
    Grow(f32),
}

/// Width and height sizing modes for a node.
#[derive(Clone, Copy)]
pub struct SizeSpec {
    pub width: SizeMode,
    pub height: SizeMode,
}

impl SizeSpec {
    pub fn new(width: SizeMode, height: SizeMode) -> Self {
        Self { width, height }
    }
}

/// Parent-relative alignment for non-participating overlay children.
#[derive(Clone, Copy)]
pub enum OverlayAlign {
    Start,
    Center,
    End,
}

#[derive(Clone, Copy)]
struct OverlayPlacement {
    horizontal: OverlayAlign,
    vertical: OverlayAlign,
}

impl OverlayPlacement {
    fn new(horizontal: OverlayAlign, vertical: OverlayAlign) -> Self {
        Self {
            horizontal,
            vertical,
        }
    }

    fn centered() -> Self {
        Self::new(OverlayAlign::Center, OverlayAlign::Center)
    }
}

#[derive(Clone, Copy)]
enum StackAxis {
    Vertical,
    Horizontal,
}

#[derive(Clone, Copy)]
struct Size2 {
    w: f32,
    h: f32,
}

impl Size2 {
    fn new(w: f32, h: f32) -> Self {
        Self {
            w: w.max(0.0),
            h: h.max(0.0),
        }
    }
}

#[derive(Clone, Copy)]
struct AxisValue {
    main: f32,
    cross: f32,
}

#[derive(Clone, Copy)]
struct AxisRect {
    main_origin: f32,
    cross_origin: f32,
    main: f32,
    cross: f32,
}

const DEBUG_GLYPH_WIDTH: f32 = 8.0;
const DEBUG_GLYPH_HEIGHT: f32 = 8.0;
const DEFAULT_FONT_POINT_SIZE: f32 = 16.0;

#[derive(Clone, Copy)]
struct TextFont {
    bytes: &'static [u8],
    point_size: f32,
}

#[derive(Clone)]
pub struct TextLabel {
    text: String,
    color: Color,
    size: SizeSpec,
    font: Option<TextFont>,
    wraps: bool,
}

impl TextLabel {
    /// Creates a text label node.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            color: Color::RGB(245, 248, 252),
            size: SizeSpec::new(SizeMode::FitContent, SizeMode::FitContent),
            font: None,
            wraps: false,
        }
    }

    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn size(mut self, width: SizeMode, height: SizeMode) -> Self {
        self.size.width = width;
        self.size.height = height;
        self
    }

    pub fn with_width(mut self, mode: SizeMode) -> Self {
        self.size.width = mode;
        self
    }

    pub fn with_height(mut self, mode: SizeMode) -> Self {
        self.size.height = mode;
        self
    }

    /// Renders this label with a TTF/OTF font from static bytes.
    pub fn with_ttf_font(mut self, font_bytes: &'static [u8], point_size: f32) -> Self {
        self.font = Some(TextFont {
            bytes: font_bytes,
            point_size: point_size.max(1.0),
        });
        self
    }

    /// Enables or disables word wrapping for TTF labels.
    pub fn with_wrap(mut self, wraps: bool) -> Self {
        self.wraps = wraps;
        self
    }

    fn intrinsic_size(&self, available: Size2) -> Size2 {
        let available = self.resolve_ttf_measure_available(available);
        if let Some(font) = self.font
            && let Ok(size) = self.measure_ttf_intrinsic_size(font, available)
        {
            return size;
        }

        Size2::new(
            self.text.chars().count() as f32 * DEBUG_GLYPH_WIDTH,
            DEBUG_GLYPH_HEIGHT,
        )
    }

    fn measure(&self, available: Size2) -> Size2 {
        let intrinsic = self.intrinsic_size(available);
        Size2::new(
            resolve_dimension(self.size.width, intrinsic.w, available.w),
            resolve_dimension(self.size.height, intrinsic.h, available.h),
        )
    }

    fn render_in_rect(&self, canvas: &mut WindowCanvas, rect: FRect) -> Result<(), String> {
        if let Some(font) = self.font
            && self.render_ttf_label(canvas, rect, font).is_ok()
        {
            return Ok(());
        }

        canvas.set_draw_color(self.color);
        canvas
            .draw_debug_text(self.text.as_str(), (rect.x, rect.y))
            .map_err(|e| e.to_string())
    }

    fn resolve_ttf_measure_available(&self, available: Size2) -> Size2 {
        let constrained_width = match self.size.width {
            SizeMode::Fixed(value) => value.max(0.0).min(available.w.max(0.0)),
            _ => available.w.max(0.0),
        };
        Size2::new(constrained_width, available.h)
    }

    fn measure_ttf_intrinsic_size(
        &self,
        font: TextFont,
        available: Size2,
    ) -> Result<Size2, String> {
        let ttf_context = ttf::init().map_err(|e| e.to_string())?;
        let font = open_ttf_font(&ttf_context, font)?;

        if self.wraps {
            let wrap_width = available.w.max(0.0).floor() as i32;
            if wrap_width > 0 {
                let surface = font
                    .render(self.text.as_str())
                    .blended_wrapped(self.color, wrap_width)
                    .map_err(|e| e.to_string())?;
                let (w, h) = surface.size();
                return Ok(Size2::new(w as f32, h as f32));
            }
        }

        let (w, h) = font
            .size_of(self.text.as_str())
            .map_err(|e| e.to_string())?;
        Ok(Size2::new(w as f32, h as f32))
    }

    fn render_ttf_label(
        &self,
        canvas: &mut WindowCanvas,
        rect: FRect,
        font: TextFont,
    ) -> Result<(), String> {
        if rect.w <= 0.0 || rect.h <= 0.0 {
            return Ok(());
        }

        let ttf_context = ttf::init().map_err(|e| e.to_string())?;
        let font = open_ttf_font(&ttf_context, font)?;
        let surface = if self.wraps {
            let wrap_width = rect.w.max(0.0).floor() as i32;
            if wrap_width > 0 {
                font.render(self.text.as_str())
                    .blended_wrapped(self.color, wrap_width)
                    .map_err(|e| e.to_string())?
            } else {
                font.render(self.text.as_str())
                    .blended(self.color)
                    .map_err(|e| e.to_string())?
            }
        } else {
            font.render(self.text.as_str())
                .blended(self.color)
                .map_err(|e| e.to_string())?
        };

        let texture_creator = canvas.texture_creator();
        let texture = texture_creator
            .create_texture_from_surface(&surface)
            .map_err(|e| e.to_string())?;
        let (surface_width, surface_height) = surface.size();
        let draw_width = (surface_width as f32).min(rect.w.max(0.0));
        let draw_height = (surface_height as f32).min(rect.h.max(0.0));
        let src_rect = FRect::new(0.0, 0.0, draw_width, draw_height);
        let dst_rect = FRect::new(rect.x, rect.y, draw_width, draw_height);

        canvas
            .copy(&texture, Some(src_rect), Some(dst_rect))
            .map_err(|e| e.to_string())
    }
}

fn open_ttf_font(
    ttf_context: &ttf::Sdl3TtfContext,
    font: TextFont,
) -> Result<ttf::Font<'static>, String> {
    let stream = IOStream::from_bytes(font.bytes).map_err(|e| e.to_string())?;
    ttf_context
        .load_font_from_iostream(stream, sanitize_font_point_size(font.point_size))
        .map_err(|e| e.to_string())
}

fn sanitize_font_point_size(point_size: f32) -> f32 {
    if point_size.is_finite() {
        point_size.max(1.0)
    } else {
        DEFAULT_FONT_POINT_SIZE
    }
}

#[derive(Clone)]
pub struct SvgIcon {
    svg_bytes: &'static [u8],
    size: SizeSpec,
    intrinsic_width: f32,
    intrinsic_height: f32,
}

impl SvgIcon {
    /// Creates an icon from SVG bytes.
    pub fn from_svg_bytes(svg_bytes: &'static [u8]) -> Self {
        Self {
            svg_bytes,
            size: SizeSpec::new(SizeMode::FitContent, SizeMode::FitContent),
            intrinsic_width: 24.0,
            intrinsic_height: 24.0,
        }
    }

    pub fn fixed_size(mut self, width: f32, height: f32) -> Self {
        self.intrinsic_width = width.max(0.0);
        self.intrinsic_height = height.max(0.0);
        self.size.width = SizeMode::Fixed(width.max(0.0));
        self.size.height = SizeMode::Fixed(height.max(0.0));
        self
    }

    pub fn size(mut self, width: SizeMode, height: SizeMode) -> Self {
        self.size.width = width;
        self.size.height = height;
        self
    }

    pub fn with_width(mut self, mode: SizeMode) -> Self {
        self.size.width = mode;
        self
    }

    pub fn with_height(mut self, mode: SizeMode) -> Self {
        self.size.height = mode;
        self
    }

    fn measure(&self, available: Size2) -> Size2 {
        Size2::new(
            resolve_dimension(self.size.width, self.intrinsic_width, available.w),
            resolve_dimension(self.size.height, self.intrinsic_height, available.h),
        )
    }

    fn render_in_rect(&self, canvas: &mut WindowCanvas, rect: FRect) -> Result<(), String> {
        if rect.w <= 0.0 || rect.h <= 0.0 {
            return Ok(());
        }

        let stream = IOStream::from_bytes(self.svg_bytes).map_err(|e| e.to_string())?;
        let surface = stream.load_typed("SVG").map_err(|e| e.to_string())?;
        let texture_creator = canvas.texture_creator();
        let texture = texture_creator
            .create_texture_from_surface(&surface)
            .map_err(|e| e.to_string())?;

        canvas
            .copy(&texture, None, Some(rect))
            .map_err(|e| e.to_string())
    }
}

#[derive(Clone)]
struct OverlayChild {
    node: LayoutNode,
    placement: OverlayPlacement,
}

#[derive(Clone)]
pub struct ColorBlock {
    background: Color,
    size: SizeSpec,
    intrinsic_width: f32,
    intrinsic_height: f32,
    overlays: Vec<OverlayChild>,
}

impl ColorBlock {
    /// Creates a simple colored rectangle with a default intrinsic size.
    pub fn new(background: Color) -> Self {
        Self {
            background,
            size: SizeSpec::new(SizeMode::FitContent, SizeMode::FitContent),
            intrinsic_width: 48.0,
            intrinsic_height: 48.0,
            overlays: Vec::new(),
        }
    }

    pub fn fixed_size(mut self, width: f32, height: f32) -> Self {
        self.intrinsic_width = width.max(0.0);
        self.intrinsic_height = height.max(0.0);
        self.size.width = SizeMode::Fixed(width.max(0.0));
        self.size.height = SizeMode::Fixed(height.max(0.0));
        self
    }

    pub fn size(mut self, width: SizeMode, height: SizeMode) -> Self {
        self.size.width = width;
        self.size.height = height;
        self
    }

    pub fn with_width(mut self, mode: SizeMode) -> Self {
        self.size.width = mode;
        self
    }

    pub fn with_height(mut self, mode: SizeMode) -> Self {
        self.size.height = mode;
        self
    }

    pub fn push_overlay(
        mut self,
        child: impl Into<LayoutNode>,
        horizontal: OverlayAlign,
        vertical: OverlayAlign,
    ) -> Self {
        self.overlays.push(OverlayChild {
            node: child.into(),
            placement: OverlayPlacement::new(horizontal, vertical),
        });
        self
    }

    pub fn push_overlay_center(self, child: impl Into<LayoutNode>) -> Self {
        self.push_overlay(child, OverlayAlign::Center, OverlayAlign::Center)
    }

    fn measure(&self, available: Size2) -> Size2 {
        Size2::new(
            resolve_dimension(self.size.width, self.intrinsic_width, available.w),
            resolve_dimension(self.size.height, self.intrinsic_height, available.h),
        )
    }

    fn render_in_rect(&self, canvas: &mut WindowCanvas, rect: FRect) -> Result<(), String> {
        canvas.set_draw_color(self.background);
        canvas.fill_rect(rect).map_err(|e| e.to_string())?;

        for overlay in &self.overlays {
            render_overlay_node(&overlay.node, overlay.placement, canvas, rect)?;
        }

        Ok(())
    }
}

#[derive(Clone, Copy)]
enum ChildLayoutMode {
    Flow,
    Overlay(OverlayPlacement),
}

impl ChildLayoutMode {
    fn is_flow(self) -> bool {
        matches!(self, ChildLayoutMode::Flow)
    }
}

#[derive(Clone)]
struct LayoutChild {
    node: LayoutNode,
    mode: ChildLayoutMode,
}

impl LayoutChild {
    fn flow(node: LayoutNode) -> Self {
        Self {
            node,
            mode: ChildLayoutMode::Flow,
        }
    }

    fn overlay(node: LayoutNode, horizontal: OverlayAlign, vertical: OverlayAlign) -> Self {
        Self {
            node,
            mode: ChildLayoutMode::Overlay(OverlayPlacement::new(horizontal, vertical)),
        }
    }
}

/// Tree node used by stack layout containers.
///
/// Public mainly to support ergonomic `push()` calls from page code.
#[derive(Clone)]
pub enum LayoutNode {
    Stack(Stack),
    Block(ColorBlock),
    Text(TextLabel),
    Icon(SvgIcon),
}

impl LayoutNode {
    fn size_spec(&self) -> SizeSpec {
        match self {
            LayoutNode::Stack(stack) => stack.size,
            LayoutNode::Block(block) => block.size,
            LayoutNode::Text(text) => text.size,
            LayoutNode::Icon(icon) => icon.size,
        }
    }

    fn measure(&self, available: Size2) -> Size2 {
        match self {
            LayoutNode::Block(block) => block.measure(available),
            LayoutNode::Stack(stack) => stack.measure(available),
            LayoutNode::Text(text) => text.measure(available),
            LayoutNode::Icon(icon) => icon.measure(available),
        }
    }

    fn render_in_rect(&self, canvas: &mut WindowCanvas, rect: FRect) -> Result<(), String> {
        match self {
            LayoutNode::Block(block) => block.render_in_rect(canvas, rect),
            LayoutNode::Stack(stack) => stack.render_in_rect(canvas, rect),
            LayoutNode::Text(text) => text.render_in_rect(canvas, rect),
            LayoutNode::Icon(icon) => icon.render_in_rect(canvas, rect),
        }
    }

    fn main_mode(&self, axis: StackAxis) -> SizeMode {
        let spec = self.size_spec();
        match axis {
            StackAxis::Vertical => spec.height,
            StackAxis::Horizontal => spec.width,
        }
    }

    fn cross_mode(&self, axis: StackAxis) -> SizeMode {
        let spec = self.size_spec();
        match axis {
            StackAxis::Vertical => spec.width,
            StackAxis::Horizontal => spec.height,
        }
    }

    fn coerce_main_grow_to_fit(&mut self, axis: StackAxis) {
        let size = match self {
            LayoutNode::Stack(stack) => &mut stack.size,
            LayoutNode::Block(block) => &mut block.size,
            LayoutNode::Text(text) => &mut text.size,
            LayoutNode::Icon(icon) => &mut icon.size,
        };

        match axis {
            StackAxis::Vertical => {
                if matches!(size.height, SizeMode::Grow(_)) {
                    size.height = SizeMode::FitContent;
                }
            }
            StackAxis::Horizontal => {
                if matches!(size.width, SizeMode::Grow(_)) {
                    size.width = SizeMode::FitContent;
                }
            }
        }
    }
}

impl From<Stack> for LayoutNode {
    fn from(value: Stack) -> Self {
        LayoutNode::Stack(value)
    }
}

impl From<ColorBlock> for LayoutNode {
    fn from(value: ColorBlock) -> Self {
        LayoutNode::Block(value)
    }
}

impl From<TextLabel> for LayoutNode {
    fn from(value: TextLabel) -> Self {
        LayoutNode::Text(value)
    }
}

impl From<SvgIcon> for LayoutNode {
    fn from(value: SvgIcon) -> Self {
        LayoutNode::Icon(value)
    }
}

#[derive(Clone)]
pub struct Stack {
    axis: StackAxis,
    background: Option<Color>,
    size: SizeSpec,
    spacing: f32,
    padding: f32,
    children: Vec<LayoutChild>,
}

impl Stack {
    fn new(axis: StackAxis, background: Option<Color>) -> Self {
        let size = match axis {
            StackAxis::Vertical => SizeSpec::new(SizeMode::FillParent, SizeMode::FitContent),
            StackAxis::Horizontal => SizeSpec::new(SizeMode::FitContent, SizeMode::FillParent),
        };

        Self {
            axis,
            background,
            size,
            spacing: 8.0,
            padding: 0.0,
            children: Vec::new(),
        }
    }

    fn with_spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing.max(0.0);
        self
    }

    fn with_padding(mut self, padding: f32) -> Self {
        self.padding = padding.max(0.0);
        self
    }

    fn with_width(mut self, mode: SizeMode) -> Self {
        self.size.width = mode;
        self
    }

    fn with_height(mut self, mode: SizeMode) -> Self {
        self.size.height = mode;
        self
    }

    fn push(mut self, child: LayoutNode) -> Self {
        self.children.push(LayoutChild::flow(child));
        self
    }

    fn push_overlay(
        mut self,
        child: LayoutNode,
        horizontal: OverlayAlign,
        vertical: OverlayAlign,
    ) -> Self {
        self.children
            .push(LayoutChild::overlay(child, horizontal, vertical));
        self
    }

    fn push_overlay_center(self, child: LayoutNode) -> Self {
        self.push_overlay(child, OverlayAlign::Center, OverlayAlign::Center)
    }

    fn measure(&self, available: Size2) -> Size2 {
        // First compute intrinsic/base size, then resolve this stack's own size mode.
        let intrinsic = self.measure_intrinsic(available);
        let has_main_grow = self.has_main_axis_grow_child();
        let resolved_main = match self.axis {
            StackAxis::Vertical => resolve_stack_main_dimension(
                self.size.height,
                intrinsic.h,
                available.h,
                has_main_grow,
            ),
            StackAxis::Horizontal => resolve_stack_main_dimension(
                self.size.width,
                intrinsic.w,
                available.w,
                has_main_grow,
            ),
        };

        match self.axis {
            StackAxis::Vertical => Size2::new(
                resolve_dimension(self.size.width, intrinsic.w, available.w),
                resolved_main,
            ),
            StackAxis::Horizontal => Size2::new(
                resolved_main,
                resolve_dimension(self.size.height, intrinsic.h, available.h),
            ),
        }
    }

    fn measure_intrinsic(&self, available: Size2) -> Size2 {
        let inner_available = Size2::new(
            (available.w - self.padding * 2.0).max(0.0),
            (available.h - self.padding * 2.0).max(0.0),
        );

        let flow_children: Vec<&LayoutChild> = self
            .children
            .iter()
            .filter(|child| child.mode.is_flow())
            .collect();

        let mut content_main = 0.0f32;
        let mut content_cross = 0.0f32;

        for child in &flow_children {
            // Base child size: coerce main-axis grow to fit-content during measure.
            let child_size = self.measure_child_base_size(&child.node, inner_available);
            let axis_size = to_axis(self.axis, child_size);
            content_main += axis_size.main;
            content_cross = content_cross.max(axis_size.cross);
        }

        if flow_children.len() > 1 {
            content_main += self.spacing * (flow_children.len() as f32 - 1.0);
        }

        from_axis(
            self.axis,
            AxisValue {
                main: content_main + self.padding * 2.0,
                cross: content_cross + self.padding * 2.0,
            },
        )
    }

    fn render_in_rect(&self, canvas: &mut WindowCanvas, rect: FRect) -> Result<(), String> {
        if let Some(color) = self.background {
            canvas.set_draw_color(color);
            canvas.fill_rect(rect).map_err(|e| e.to_string())?;
        }

        let padding = self.padding;
        let inner_rect = FRect::new(
            rect.x + padding,
            rect.y + padding,
            (rect.w - padding * 2.0).max(0.0),
            (rect.h - padding * 2.0).max(0.0),
        );

        let flow_children: Vec<&LayoutChild> = self
            .children
            .iter()
            .filter(|child| child.mode.is_flow())
            .collect();

        if !flow_children.is_empty() {
            let inner_size = Size2::new(inner_rect.w, inner_rect.h);
            let gaps = if flow_children.len() > 1 {
                self.spacing * (flow_children.len() as f32 - 1.0)
            } else {
                0.0
            };

            let mut measured_sizes: Vec<Size2> = Vec::with_capacity(flow_children.len());
            let mut total_base_main = 0.0f32;
            let mut total_grow_weight = 0.0f32;

            for child in &flow_children {
                // Measure base size once and re-use it for final arrangement below.
                let measured = self.measure_child_base_size(&child.node, inner_size);
                total_base_main += to_axis(self.axis, measured).main;
                if let SizeMode::Grow(weight) = child.node.main_mode(self.axis)
                    && weight > 0.0
                {
                    total_grow_weight += weight;
                }
                measured_sizes.push(measured);
            }

            let axis_rect = to_axis_rect(self.axis, inner_rect);
            let available_main = axis_rect.main;
            let extra_main = (available_main - total_base_main - gaps).max(0.0);

            let main_end = axis_rect.main_origin + axis_rect.main;
            let mut cursor_main = axis_rect.main_origin;

            for (index, child) in flow_children.iter().enumerate() {
                let child_size = measured_sizes[index];
                let current_axis = to_axis(self.axis, child_size);

                let requested_main = match child.node.main_mode(self.axis) {
                    SizeMode::Grow(weight) if weight > 0.0 && total_grow_weight > 0.0 => {
                        current_axis.main + extra_main * (weight / total_grow_weight)
                    }
                    SizeMode::Fixed(value) => value.max(0.0),
                    SizeMode::FillParent => available_main,
                    _ => current_axis.main,
                };
                // Hard bound against parent overflow: children cannot exceed remaining space.
                let remaining_main = (main_end - cursor_main).max(0.0);
                let child_main = requested_main.max(0.0).min(remaining_main);

                let available_cross = axis_rect.cross;
                let child_cross = match child.node.cross_mode(self.axis) {
                    SizeMode::FillParent | SizeMode::Grow(_) => available_cross,
                    SizeMode::Fixed(value) => value.max(0.0).min(available_cross),
                    _ => current_axis.cross.min(available_cross),
                };

                let child_rect = from_axis_rect(
                    self.axis,
                    cursor_main,
                    axis_rect.cross_origin,
                    child_main.max(0.0),
                    child_cross.max(0.0),
                );

                child.node.render_in_rect(canvas, child_rect)?;

                cursor_main += child_main;
                if index + 1 < flow_children.len() {
                    // Preserve spacing without stepping past the parent edge.
                    let remaining_after_child = (main_end - cursor_main).max(0.0);
                    cursor_main += self.spacing.min(remaining_after_child);
                }
            }
        }

        for child in &self.children {
            let placement = match child.mode {
                ChildLayoutMode::Overlay(placement) => placement,
                ChildLayoutMode::Flow => continue,
            };
            render_overlay_node(&child.node, placement, canvas, inner_rect)?;
        }

        Ok(())
    }

    fn measure_child_base_size(&self, child: &LayoutNode, available: Size2) -> Size2 {
        let mut node = child.clone();
        node.coerce_main_grow_to_fit(self.axis);
        node.measure(available)
    }

    fn has_main_axis_grow_child(&self) -> bool {
        self.children
            .iter()
            .filter(|child| child.mode.is_flow())
            .any(|child| match child.node.main_mode(self.axis) {
                SizeMode::Grow(weight) => weight > 0.0,
                _ => false,
            })
    }
}

#[derive(Clone)]
pub struct VStack {
    inner: Stack,
}

impl VStack {
    /// Creates a vertical stack with a background fill.
    pub fn new(background: Color) -> Self {
        Self {
            inner: Stack::new(StackAxis::Vertical, Some(background)),
        }
    }

    pub fn with_spacing(mut self, spacing: f32) -> Self {
        self.inner = self.inner.with_spacing(spacing);
        self
    }

    pub fn with_padding(mut self, padding: f32) -> Self {
        self.inner = self.inner.with_padding(padding);
        self
    }

    pub fn with_width(mut self, mode: SizeMode) -> Self {
        self.inner = self.inner.with_width(mode);
        self
    }

    pub fn with_height(mut self, mode: SizeMode) -> Self {
        self.inner = self.inner.with_height(mode);
        self
    }

    pub fn fill_both(self) -> Self {
        self.with_width(SizeMode::FillParent)
            .with_height(SizeMode::FillParent)
    }

    pub fn push(mut self, child: impl Into<LayoutNode>) -> Self {
        self.inner = self.inner.push(child.into());
        self
    }

    pub fn push_overlay(
        mut self,
        child: impl Into<LayoutNode>,
        horizontal: OverlayAlign,
        vertical: OverlayAlign,
    ) -> Self {
        self.inner = self.inner.push_overlay(child.into(), horizontal, vertical);
        self
    }

    pub fn push_overlay_center(mut self, child: impl Into<LayoutNode>) -> Self {
        self.inner = self.inner.push_overlay_center(child.into());
        self
    }

    /// Renders this stack into the full viewport rectangle.
    pub fn render(self, canvas: &mut WindowCanvas, viewport: Viewport) -> Result<(), String> {
        let rect = FRect::new(0.0, 0.0, viewport.width as f32, viewport.height as f32);
        self.inner.render_in_rect(canvas, rect)
    }
}

impl From<VStack> for LayoutNode {
    fn from(value: VStack) -> Self {
        value.inner.into()
    }
}

#[derive(Clone)]
pub struct HStack {
    inner: Stack,
}

impl HStack {
    /// Creates a horizontal stack with a background fill.
    pub fn new(background: Color) -> Self {
        Self {
            inner: Stack::new(StackAxis::Horizontal, Some(background)),
        }
    }

    pub fn with_spacing(mut self, spacing: f32) -> Self {
        self.inner = self.inner.with_spacing(spacing);
        self
    }

    pub fn with_padding(mut self, padding: f32) -> Self {
        self.inner = self.inner.with_padding(padding);
        self
    }

    pub fn with_width(mut self, mode: SizeMode) -> Self {
        self.inner = self.inner.with_width(mode);
        self
    }

    pub fn with_height(mut self, mode: SizeMode) -> Self {
        self.inner = self.inner.with_height(mode);
        self
    }

    pub fn fill_both(self) -> Self {
        self.with_width(SizeMode::FillParent)
            .with_height(SizeMode::FillParent)
    }

    pub fn push(mut self, child: impl Into<LayoutNode>) -> Self {
        self.inner = self.inner.push(child.into());
        self
    }

    pub fn push_overlay(
        mut self,
        child: impl Into<LayoutNode>,
        horizontal: OverlayAlign,
        vertical: OverlayAlign,
    ) -> Self {
        self.inner = self.inner.push_overlay(child.into(), horizontal, vertical);
        self
    }

    pub fn push_overlay_center(mut self, child: impl Into<LayoutNode>) -> Self {
        self.inner = self.inner.push_overlay_center(child.into());
        self
    }
}

impl From<HStack> for LayoutNode {
    fn from(value: HStack) -> Self {
        value.inner.into()
    }
}

fn render_overlay_node(
    node: &LayoutNode,
    placement: OverlayPlacement,
    canvas: &mut WindowCanvas,
    parent_rect: FRect,
) -> Result<(), String> {
    let available = Size2::new(parent_rect.w, parent_rect.h);
    let measured = node.measure(available);
    let child_size = resolve_overlay_size(node.size_spec(), measured, available);
    let child_rect = align_overlay_rect(parent_rect, child_size, placement);
    node.render_in_rect(canvas, child_rect)
}

fn resolve_overlay_size(size_spec: SizeSpec, measured: Size2, available: Size2) -> Size2 {
    Size2::new(
        resolve_overlay_dimension(size_spec.width, measured.w, available.w),
        resolve_overlay_dimension(size_spec.height, measured.h, available.h),
    )
}

fn resolve_overlay_dimension(mode: SizeMode, measured: f32, available: f32) -> f32 {
    match mode {
        SizeMode::FillParent => available.max(0.0),
        SizeMode::Fixed(value) => value.max(0.0).min(available.max(0.0)),
        // Overlay children are non-participating, so grow has no sibling context.
        SizeMode::Grow(_) | SizeMode::FitContent => measured.max(0.0).min(available.max(0.0)),
    }
}

fn align_overlay_rect(parent_rect: FRect, child_size: Size2, placement: OverlayPlacement) -> FRect {
    let x = align_origin(
        parent_rect.x,
        parent_rect.w,
        child_size.w,
        placement.horizontal,
    );
    let y = align_origin(
        parent_rect.y,
        parent_rect.h,
        child_size.h,
        placement.vertical,
    );
    FRect::new(x, y, child_size.w, child_size.h)
}

fn align_origin(origin: f32, parent_size: f32, child_size: f32, align: OverlayAlign) -> f32 {
    let remaining = (parent_size - child_size).max(0.0);
    match align {
        OverlayAlign::Start => origin,
        OverlayAlign::Center => origin + remaining * 0.5,
        OverlayAlign::End => origin + remaining,
    }
}

fn resolve_dimension(mode: SizeMode, intrinsic: f32, available: f32) -> f32 {
    let available = available.max(0.0);
    let intrinsic = intrinsic.max(0.0);

    match mode {
        SizeMode::FitContent => intrinsic.min(available),
        // Fill is an arrange-time concern; use intrinsic as the measurement baseline.
        SizeMode::FillParent => intrinsic.min(available),
        SizeMode::Fixed(value) => value.max(0.0).min(available),
        // Grow also participates at arrange-time, not measure-time.
        SizeMode::Grow(_) => intrinsic.min(available),
    }
}

fn resolve_stack_main_dimension(
    mode: SizeMode,
    intrinsic: f32,
    available: f32,
    has_main_grow_child: bool,
) -> f32 {
    match mode {
        // Fit-content stacks with grow children need full available main-axis space
        // so weighted distribution can happen in arrange.
        SizeMode::FitContent if has_main_grow_child => available.max(0.0),
        _ => resolve_dimension(mode, intrinsic, available),
    }
}

fn to_axis(axis: StackAxis, size: Size2) -> AxisValue {
    match axis {
        StackAxis::Vertical => AxisValue {
            main: size.h,
            cross: size.w,
        },
        StackAxis::Horizontal => AxisValue {
            main: size.w,
            cross: size.h,
        },
    }
}

fn from_axis(axis: StackAxis, value: AxisValue) -> Size2 {
    match axis {
        StackAxis::Vertical => Size2::new(value.cross, value.main),
        StackAxis::Horizontal => Size2::new(value.main, value.cross),
    }
}

fn to_axis_rect(axis: StackAxis, rect: FRect) -> AxisRect {
    match axis {
        StackAxis::Vertical => AxisRect {
            main_origin: rect.y,
            cross_origin: rect.x,
            main: rect.h,
            cross: rect.w,
        },
        StackAxis::Horizontal => AxisRect {
            main_origin: rect.x,
            cross_origin: rect.y,
            main: rect.w,
            cross: rect.h,
        },
    }
}

fn from_axis_rect(
    axis: StackAxis,
    main_origin: f32,
    cross_origin: f32,
    main: f32,
    cross: f32,
) -> FRect {
    match axis {
        StackAxis::Vertical => FRect::new(cross_origin, main_origin, cross, main),
        StackAxis::Horizontal => FRect::new(main_origin, cross_origin, main, cross),
    }
}
