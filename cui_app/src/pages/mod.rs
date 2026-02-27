#[allow(dead_code)]
pub mod layout;
pub mod test;

use sdl3::render::WindowCanvas;

pub trait AppPage {
    fn update(&mut self, dt_seconds: f32);
    fn render(&self, canvas: &mut WindowCanvas, viewport: Viewport) -> Result<(), String>;
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct Viewport {
    pub width: u32,
    pub height: u32,
}

impl Viewport {
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum PageId {
    Test,
}

impl PageId {
    pub fn all() -> &'static [PageId] {
        &[PageId::Test]
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            PageId::Test => "test",
        }
    }

    pub fn from_id(value: &str) -> Result<Self, String> {
        match value {
            "test" => Ok(PageId::Test),
            _ => Err(format!("Unknown page id: {}", value)),
        }
    }
}

pub struct PageManager {
    active_page: Box<dyn AppPage>,
}

impl PageManager {
    pub fn new(initial_page_id: PageId) -> Self {
        Self {
            active_page: build_page(initial_page_id),
        }
    }

    pub fn active_page_id(&self) -> PageId {
        PageId::Test
    }

    pub fn update(&mut self, dt_seconds: f32) {
        self.active_page.update(dt_seconds);
    }

    pub fn render(&self, canvas: &mut WindowCanvas, viewport: Viewport) -> Result<(), String> {
        self.active_page.render(canvas, viewport)
    }
}

fn build_page(page_id: PageId) -> Box<dyn AppPage> {
    match page_id {
        PageId::Test => Box::new(test::TestPage::new()),
    }
}
