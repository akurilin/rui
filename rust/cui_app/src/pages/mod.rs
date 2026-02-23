pub mod corners;
#[allow(dead_code)]
pub mod layout;
pub mod showcase;
pub mod test;
pub mod todo;

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
    Todo,
    Corners,
    Showcase,
    Test,
}

impl PageId {
    pub fn all() -> &'static [PageId] {
        &[
            PageId::Todo,
            PageId::Corners,
            PageId::Showcase,
            PageId::Test,
        ]
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            PageId::Todo => "todo",
            PageId::Corners => "corners",
            PageId::Showcase => "showcase",
            PageId::Test => "test",
        }
    }

    pub fn from_id(value: &str) -> Result<Self, String> {
        match value {
            "todo" => Ok(PageId::Todo),
            "corners" => Ok(PageId::Corners),
            "showcase" => Ok(PageId::Showcase),
            "test" => Ok(PageId::Test),
            _ => Err(format!("Unknown page id: {}", value)),
        }
    }
}

pub struct PageManager {
    active_id: PageId,
    active_page: Box<dyn AppPage>,
}

impl PageManager {
    pub fn new(initial_page_id: PageId) -> Self {
        Self {
            active_id: initial_page_id,
            active_page: build_page(initial_page_id),
        }
    }

    pub fn active_page_id(&self) -> PageId {
        self.active_id
    }

    pub fn switch_to(&mut self, page_id: PageId) {
        if self.active_id == page_id {
            return;
        }

        self.active_id = page_id;
        self.active_page = build_page(page_id);
    }

    pub fn cycle_next(&mut self) {
        let page_ids = PageId::all();
        let current = page_ids
            .iter()
            .position(|id| *id == self.active_id)
            .unwrap_or(0);
        let next = (current + 1) % page_ids.len();
        self.switch_to(page_ids[next]);
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
        PageId::Todo => Box::new(todo::TodoPage::new()),
        PageId::Corners => Box::new(corners::CornersPage::new()),
        PageId::Showcase => Box::new(showcase::ShowcasePage::new()),
        PageId::Test => Box::new(test::TestPage::new()),
    }
}
