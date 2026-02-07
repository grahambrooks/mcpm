use std::io;
use std::time::Duration;

use clap::Parser;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use mcpm::App;

#[derive(Parser, Debug)]
#[command(name = "mcpm")]
#[command(about = "MCP Server Manager - Manage MCP servers across IDEs")]
#[command(version)]
struct Args {
    /// Enable debug logging
    #[arg(short, long)]
    debug: bool,

    /// Jump directly to a specific MCP server by name
    #[arg(short, long)]
    server: Option<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // Set up logging
    if args.debug {
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| "mcpm=debug".into()),
            )
            .with(tracing_subscriber::fmt::layer().with_target(false))
            .init();
    }

    // Set up terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create and initialize app
    let mut app = App::new(args.server);
    if let Err(e) = app.init().await {
        // Restore terminal before printing error
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;
        eprintln!("Initialization error: {}", e);
        return Err(e.into());
    }

    // Start registry fetch in the background (non-blocking)
    app.start_registry_fetch();

    // Main loop
    let result = run_app(&mut terminal, &mut app).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(e) = result {
        eprintln!("Application error: {}", e);
        return Err(e.into());
    }

    Ok(())
}

async fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
) -> mcpm::Result<()> {
    loop {
        // Poll for completed async tasks
        app.poll_tasks();

        // Draw UI
        terminal.draw(|f| mcpm::view::render(f, &app.state))?;

        // Handle events with timeout for async operations
        if event::poll(Duration::from_millis(100))? {
            let event = event::read()?;
            app.handle_event(event).await?;
        }

        if app.should_quit() {
            break;
        }
    }

    Ok(())
}
