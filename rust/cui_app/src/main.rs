mod app;
mod pages;

fn main() -> Result<(), String> {
    match app::AppConfig::parse_from_env()? {
        app::StartupDecision::Run(config) => app::run(config),
        app::StartupDecision::ExitSuccess => Ok(()),
    }
}
