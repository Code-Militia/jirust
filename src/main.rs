mod app;
mod components;
mod jira;
mod jtui;
mod event;

use anyhow;
use app::App;
use chrono;
use event::event::Event;
use fern;
use std::io;
use tui::{
    backend::CrosstermBackend,
    Terminal,
};

#[derive(Copy, Clone, Debug)]
enum MenuItem {
    Home,
    Projects,
    Issues,
}

impl From<MenuItem> for usize {
    fn from(input: MenuItem) -> usize {
        match input {
            MenuItem::Home => 0,
            MenuItem::Projects => 1,
            MenuItem::Issues => 2,
        }
    }
}

fn setup_logger() -> Result<(), fern::InitError> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Debug)
        .chain(fern::log_file("output.log")?)
        .apply()?;
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    setup_logger()?;
    let mut app: App = App::new().await?;
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let events = event::event::Events::new(250);

    terminal.clear()?;

    loop {
        terminal.draw(|f| {
            if let Err(err) = app.draw(f) {
                std::process::exit(1);
            }
        })?;
        // match events.next()? {
        //     Event::Input(key) => match app. {
        //
        //     }
        // }
        {
            break;
        }
    }

    Ok(()) 
}
