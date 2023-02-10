// mod app;
// mod components;
// mod config;
// mod event;
// mod jira;
// mod jtui;

use serde::Deserialize;
use serde::Serialize;
use surrealdb::Surreal;
use surrealdb::engine::any::connect;
use surrealdb::engine::any::Any;
use anyhow;
// use app::App;
use chrono;
// use event::event::Event;
use fern;
use log::info;
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

#[derive(Serialize, Deserialize, Debug)]
struct Settings {
    active: bool,
    marketing: bool,
}

#[derive(Serialize, Deserialize, Debug)]
struct User {
    name: String,
    settings: Settings,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // TODO: Use sqlite instead of surrealdb, since surrealdb does not support memory databases
    // TODO: Look into https://crates.io/crates/rusqlite
    setup_logger()?;
    let db = connect("memory").await?;
    db.use_ns("noc").use_db("database").await?;
    let record: User = db.create(("user", "tobie"))
    .content(User {
        name: "Tobie".to_string(),
        settings: Settings {
            active: true,
            marketing: true,
        },
    })
    .await?;

    let user: Vec<User> = db.select("user").await?;
    info!("{user:?}");
    // setup_logger()?;
    // let mut app: App = App::new().await?;
    // let stdout = io::stdout();
    // let backend = CrosstermBackend::new(stdout);
    // let mut terminal = Terminal::new(backend)?;
    // let events = event::event::Events::new(250);
    //
    // terminal.clear()?;
    //
    // loop {
    //     terminal.draw(|f| {
    //         if let Err(err) = app.draw(f) {
    //             std::process::exit(1);
    //         }
    //     })?;
    //     // match events.next()? {
    //     //     Event::Input(key) => match app. {
    //     //
    //     //     }
    //     // }
    //     {
    //         break;
    //     }
    // }
    //
    Ok(()) 
}
