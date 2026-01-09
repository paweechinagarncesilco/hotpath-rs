mod app;
mod constants;
#[cfg(feature = "hotpath")]
pub mod demo;
mod events;
mod http_worker;
mod input;
mod views;
mod widgets;

use app::App;
use clap::Parser;
use eyre::Result;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt};

fn init_logging() {
    let offset = time::UtcOffset::current_local_offset().unwrap_or(time::UtcOffset::UTC);
    let time_format =
        time::format_description::parse("[year]-[month]-[day]T[hour]:[minute]:[second]").unwrap();
    let timer = tracing_subscriber::fmt::time::OffsetTime::new(offset, time_format);
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("error"));

    std::fs::create_dir_all("log").expect("failed to create log directory");
    let log_file = std::fs::File::create("log/development.log").expect("failed to create log file");
    let file_layer = fmt::layer()
        .with_writer(log_file)
        .with_ansi(false)
        .with_timer(timer)
        .with_target(false)
        .with_thread_ids(false);

    tracing_subscriber::registry()
        .with(env_filter)
        .with(file_layer)
        .init();
}

#[derive(Debug, Parser)]
pub struct ConsoleArgs {
    #[arg(
        long,
        default_value_t = 6770,
        help = "Port where the metrics HTTP server is running"
    )]
    pub metrics_port: u16,

    #[arg(long, default_value_t = 500, help = "Refresh interval in milliseconds")]
    pub refresh_interval: u64,
}

#[hotpath::measure_all]
impl ConsoleArgs {
    pub fn run(&self) -> Result<()> {
        init_logging();

        #[cfg(feature = "hotpath")]
        demo::init();

        let mut app = App::new(self.metrics_port, self.refresh_interval);

        // Use modern ratatui initialization
        let mut terminal = ratatui::init();

        let app_result = app.run(&mut terminal);

        // Use modern ratatui restoration
        ratatui::restore();

        app_result.map_err(|e| eyre::eyre!("TUI error: {}", e))
    }
}
