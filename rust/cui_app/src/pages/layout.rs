use crate::pages::Viewport;
use sdl3::pixels::Color;
use sdl3::render::{FRect, WindowCanvas};

#[derive(Clone, Copy)]
pub enum SizeMode {
    FitContent,
    FillParent,
    Fixed(f32),
    Grow(f32),
}

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

#[derive(Clone)]
pub struct ColorBlock {
    background: Color,
    size: SizeSpec,
    intrinsic_width: f32,
    intrinsic_height: f32,
}

impl ColorBlock {
    pub fn new(background: Color) -> Self {
        Self {
            background,
            size: SizeSpec::new(SizeMode::FitContent, SizeMode::FitContent),
            intrinsic_width: 48.0,
            intrinsic_height: 48.0,
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
}

#[derive(Clone)]
pub enum LayoutNode {
    Stack(Stack),
    Block(ColorBlock),
}

impl LayoutNode {
    fn size_spec(&self) -> SizeSpec {
        match self {
            LayoutNode::Stack(stack) => stack.size,
            LayoutNode::Block(block) => block.size,
        }
    }

    fn measure(&self, available: Size2) -> Size2 {
        match self {
            LayoutNode::Block(block) => block.measure(available),
            LayoutNode::Stack(stack) => stack.measure(available),
        }
    }

    fn render_in_rect(&self, canvas: &mut WindowCanvas, rect: FRect) -> Result<(), String> {
        match self {
            LayoutNode::Block(block) => block.render_in_rect(canvas, rect),
            LayoutNode::Stack(stack) => stack.render_in_rect(canvas, rect),
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

impl From<ColorBlock> for LayoutNode {
    fn from(value: ColorBlock) -> Self {
        LayoutNode::Block(value)
    }
}

#[derive(Clone)]
struct StackChild {
    node: LayoutNode,
}

#[derive(Clone)]
pub struct Stack {
    axis: StackAxis,
    background: Option<Color>,
    size: SizeSpec,
    spacing: f32,
    padding: f32,
    children: Vec<StackChild>,
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
        self.children.push(StackChild { node: child });
        self
    }

    fn measure(&self, available: Size2) -> Size2 {
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
            StackAxis::Vertical => {
                let measured = Size2::new(
                    resolve_dimension(self.size.width, intrinsic.w, available.w),
                    resolved_main,
                );
                measured
            }
            StackAxis::Horizontal => {
                let measured = Size2::new(
                    resolved_main,
                    resolve_dimension(self.size.height, intrinsic.h, available.h),
                );
                measured
            }
        }
    }

    fn measure_intrinsic(&self, available: Size2) -> Size2 {
        let inner_available = Size2::new(
            (available.w - self.padding * 2.0).max(0.0),
            (available.h - self.padding * 2.0).max(0.0),
        );
        let spacing = self.spacing;

        let mut content_main = 0.0f32;
        let mut content_cross = 0.0f32;
        let mut measured_children = 0usize;

        for child in &self.children {
            let child_size = self.measure_child_base_size(&child.node, inner_available);
            let axis_size = to_axis(self.axis, child_size);
            content_main += axis_size.main;
            content_cross = content_cross.max(axis_size.cross);
            measured_children += 1;
        }

        if measured_children > 1 {
            content_main += spacing * (measured_children as f32 - 1.0);
        }

        let total = AxisValue {
            main: content_main + self.padding * 2.0,
            cross: content_cross + self.padding * 2.0,
        };

        from_axis(self.axis, total)
    }

    fn render_in_rect(&self, canvas: &mut WindowCanvas, rect: FRect) -> Result<(), String> {
        if let Some(color) = self.background {
            canvas.set_draw_color(color);
            canvas.fill_rect(rect).map_err(|e| e.to_string())?;
        }

        if self.children.is_empty() {
            return Ok(());
        }

        let padding = self.padding;
        let inner_rect = FRect::new(
            rect.x + padding,
            rect.y + padding,
            (rect.w - padding * 2.0).max(0.0),
            (rect.h - padding * 2.0).max(0.0),
        );

        let inner_size = Size2::new(inner_rect.w, inner_rect.h);
        let spacing = self.spacing;
        let gaps = if self.children.len() > 1 {
            spacing * (self.children.len() as f32 - 1.0)
        } else {
            0.0
        };

        let mut measured_sizes: Vec<Size2> = Vec::with_capacity(self.children.len());
        let mut total_base_main = 0.0f32;
        let mut total_grow_weight = 0.0f32;

        for child in &self.children {
            let measured = self.measure_child_base_size(&child.node, inner_size);
            total_base_main += to_axis(self.axis, measured).main;
            if let SizeMode::Grow(weight) = child.node.main_mode(self.axis) {
                if weight > 0.0 {
                    total_grow_weight += weight;
                }
            }
            measured_sizes.push(measured);
        }

        let available_main = to_axis_rect(self.axis, inner_rect).main;
        let extra_main = (available_main - total_base_main - gaps).max(0.0);

        let mut cursor_main = to_axis_rect(self.axis, inner_rect).main_origin;

        for (index, child) in self.children.iter().enumerate() {
            let child_size = measured_sizes[index];
            let current_axis = to_axis(self.axis, child_size);

            let child_main = match child.node.main_mode(self.axis) {
                SizeMode::Grow(weight) if weight > 0.0 && total_grow_weight > 0.0 => {
                    current_axis.main + extra_main * (weight / total_grow_weight)
                }
                SizeMode::Fixed(value) => value.max(0.0),
                SizeMode::FillParent => available_main,
                _ => current_axis.main,
            };

            let available_cross = to_axis_rect(self.axis, inner_rect).cross;
            let child_cross = match child.node.cross_mode(self.axis) {
                SizeMode::FillParent | SizeMode::Grow(_) => available_cross,
                SizeMode::Fixed(value) => value.max(0.0).min(available_cross),
                _ => current_axis.cross.min(available_cross),
            };

            let child_rect = from_axis_rect(
                self.axis,
                cursor_main,
                to_axis_rect(self.axis, inner_rect).cross_origin,
                child_main.max(0.0),
                child_cross.max(0.0),
            );

            child.node.render_in_rect(canvas, child_rect)?;

            cursor_main += child_main;
            if index + 1 < self.children.len() {
                cursor_main += spacing;
            }
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
            .any(|child| match child.node.main_mode(self.axis) {
                SizeMode::Grow(weight) => weight > 0.0,
                _ => false,
            })
    }
}

impl From<Stack> for LayoutNode {
    fn from(value: Stack) -> Self {
        LayoutNode::Stack(value)
    }
}

#[derive(Clone)]
pub struct VStack {
    inner: Stack,
}

impl VStack {
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
}

impl From<HStack> for LayoutNode {
    fn from(value: HStack) -> Self {
        value.inner.into()
    }
}

#[derive(Clone, Copy)]
struct AxisRect {
    main_origin: f32,
    cross_origin: f32,
    main: f32,
    cross: f32,
}

fn resolve_dimension(mode: SizeMode, intrinsic: f32, available: f32) -> f32 {
    match mode {
        SizeMode::FitContent => intrinsic.min(available.max(0.0)),
        SizeMode::FillParent => available.max(0.0),
        SizeMode::Fixed(value) => value.max(0.0).min(available.max(0.0)),
        SizeMode::Grow(_) => available.max(0.0),
    }
}

fn resolve_stack_main_dimension(
    mode: SizeMode,
    intrinsic: f32,
    available: f32,
    has_main_grow_child: bool,
) -> f32 {
    match mode {
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

impl ColorBlock {
    fn measure(&self, available: Size2) -> Size2 {
        Size2::new(
            resolve_dimension(self.size.width, self.intrinsic_width, available.w),
            resolve_dimension(self.size.height, self.intrinsic_height, available.h),
        )
    }

    fn render_in_rect(&self, canvas: &mut WindowCanvas, rect: FRect) -> Result<(), String> {
        canvas.set_draw_color(self.background);
        canvas.fill_rect(rect).map_err(|e| e.to_string())
    }
}
