use crate::pages::{PageId, PageManager, Viewport};
use sdl3::event::{Event, WindowEvent};
use sdl3::keyboard::Keycode;
use sdl3::pixels::Color;
use sdl3::render::WindowCanvas;
use sdl3::sys::render::SDL_LOGICAL_PRESENTATION_LETTERBOX;
use std::env;
use std::time::{Duration, Instant};

const DEFAULT_WINDOW_WIDTH: u32 = 2304;
const DEFAULT_WINDOW_HEIGHT: u32 = 1296;
const MIN_WINDOW_WIDTH: u32 = 640;
const MIN_WINDOW_HEIGHT: u32 = 480;

pub struct AppConfig {
    pub width: u32,
    pub height: u32,
    pub page_id: PageId,
}

pub enum StartupDecision {
    Run(AppConfig),
    ExitSuccess,
}

impl AppConfig {
    pub fn parse_from_env() -> Result<StartupDecision, String> {
        let mut config = AppConfig {
            width: DEFAULT_WINDOW_WIDTH,
            height: DEFAULT_WINDOW_HEIGHT,
            page_id: PageId::Test,
        };

        let args: Vec<String> = env::args().collect();
        let mut index = 1usize;

        while index < args.len() {
            let option = args[index].as_str();
            match option {
                "--help" => {
                    print_help(&args[0]);
                    return Ok(StartupDecision::ExitSuccess);
                }
                "--page" => {
                    index += 1;
                    if index >= args.len() {
                        return Err("Missing value for --page".to_string());
                    }
                    config.page_id = PageId::from_id(&args[index])?;
                }
                "-w" | "--width" => {
                    index += 1;
                    if index >= args.len() {
                        return Err(format!("Missing value for {}", option));
                    }
                    config.width = parse_positive_u32(option, &args[index])?;
                }
                "-h" | "--height" => {
                    index += 1;
                    if index >= args.len() {
                        return Err(format!("Missing value for {}", option));
                    }
                    config.height = parse_positive_u32(option, &args[index])?;
                }
                _ => {
                    return Err(format!("Unknown option: {}", option));
                }
            }

            index += 1;
        }

        Ok(StartupDecision::Run(config))
    }
}

pub fn run(config: AppConfig) -> Result<(), String> {
    let sdl = sdl3::init().map_err(|e| e.to_string())?;
    let video = sdl.video().map_err(|e| e.to_string())?;

    let mut window_builder = video.window("CUI - Rust SDL3 Prototype", config.width, config.height);
    window_builder.position_centered().resizable();

    let mut window = window_builder.build().map_err(|e| e.to_string())?;
    window
        .set_minimum_size(MIN_WINDOW_WIDTH, MIN_WINDOW_HEIGHT)
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas();
    apply_logical_size(&mut canvas, config.width, config.height)?;

    let mut event_pump = sdl.event_pump().map_err(|e| e.to_string())?;
    let mut page_manager = PageManager::new(config.page_id);
    let mut viewport = Viewport::new(config.width, config.height);
    let mut previous_frame = Instant::now();

    let mut running = true;
    while running {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    running = false;
                }
                Event::Window {
                    win_event: WindowEvent::Resized(width, height),
                    ..
                } => {
                    if width > 0 && height > 0 {
                        viewport = Viewport::new(width as u32, height as u32);
                        apply_logical_size(&mut canvas, viewport.width, viewport.height)?;
                    }
                }
                _ => {}
            }
        }

        let now = Instant::now();
        let dt = now.duration_since(previous_frame).as_secs_f32();
        previous_frame = now;

        page_manager.update(dt);
        page_manager.render(&mut canvas, viewport)?;
        render_overlay(&mut canvas, page_manager.active_page_id())?;
        canvas.present();

        std::thread::sleep(Duration::from_millis(16));
    }

    Ok(())
}

fn parse_positive_u32(option: &str, value: &str) -> Result<u32, String> {
    let parsed = value
        .parse::<u32>()
        .map_err(|_| format!("Invalid value for {}: {}", option, value))?;
    if parsed == 0 {
        return Err(format!("Invalid value for {}: {}", option, value));
    }
    Ok(parsed)
}

fn apply_logical_size(canvas: &mut WindowCanvas, width: u32, height: u32) -> Result<(), String> {
    canvas
        .set_logical_size(width, height, SDL_LOGICAL_PRESENTATION_LETTERBOX)
        .map_err(|e| e.to_string())
}

fn render_overlay(canvas: &mut WindowCanvas, page_id: PageId) -> Result<(), String> {
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas
        .draw_debug_text(
            &format!("page={} | layout test mode | [esc] quit", page_id.as_str()),
            (20.0, 16.0),
        )
        .map_err(|e| e.to_string())
}

fn print_help(program_name: &str) {
    println!(
        "Usage: {} [--page <id>] [-w|--width <width>] [-h|--height <height>] [--help]",
        program_name
    );
    println!("Options:");
    println!("      --page <id>        Select startup page id.");
    println!("  -w, --width <width>    Set startup window width in pixels.");
    println!("  -h, --height <height>  Set startup window height in pixels.");
    println!("      --help             Show this help message.");
    println!("Available page ids:");
    for page_id in PageId::all() {
        println!("  {}", page_id.as_str());
    }
}
