mod jira;
mod jtui;

use chrono;
use crossterm::event::{self, Event as CEvent, KeyCode};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use fern;
use jira::auth::jira_authentication;
use jtui::issues::render_issues;
use jira::issue;
use jira::projects;
use jtui::home::render_home;
use jtui::projects::render_projects;
use std::io;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};
use surrealdb::{Datastore, Session};
use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, ListState, Paragraph},
    Terminal,
};

enum Event<I> {
    Input(I),
    Tick,
}

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

pub type DB = (Datastore, Session);

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_logger().expect("logger should be setup");

    let db: &DB = &(
        Datastore::new("memory").await?,
        Session::for_db("jira", "jira"),
    );
    let auth = jira_authentication();
    let jira_projects = projects::JiraProjects {
        auth: &auth,
        db_connection: &db,
    };
    jira_projects.save_jira_projects().await?;
    let projects_list = jira_projects
        .get_jira_projects()
        .await
        .expect("failed to get jira projects list");

    let jira_issues = issue::JiraIssues {
        auth: &auth,
        db_connection: &db,
    };
    enable_raw_mode().expect("can run in raw mode");
    let (tx, rx) = mpsc::channel();
    let tick_rate = Duration::from_millis(200);
    // TODO: Need to understand this better
    thread::spawn(move || {
        let mut last_tick = Instant::now();
        loop {
            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));

            if event::poll(timeout).expect("poll works") {
                if let CEvent::Key(key) = event::read().expect("can read events") {
                    tx.send(Event::Input(key)).expect("can send events");
                }
            }

            if last_tick.elapsed() >= tick_rate {
                if let Ok(_) = tx.send(Event::Tick) {
                    last_tick = Instant::now();
                }
            }
        }
    });

    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let mut active_menu_item = MenuItem::Home;
    let mut project_list_state = ListState::default();
    project_list_state.select(Some(0));

    let mut issues_list: Vec<String> = Vec::new();
    let mut issues_list_state = ListState::default();
    issues_list_state.select(Some(0));

    loop {
        terminal.draw(|rect| {
            let size = rect.size();
            let pane = Layout::default()
                .direction(Direction::Vertical)
                .margin(0)
                .constraints(
                    [
                        Constraint::Length(0),
                        // Constraint::Length(3),
                        // Constraint::Min(2),
                        Constraint::Min(0),
                        // Constraint::Length(3),
                        Constraint::Length(3),
                    ]
                    .as_ref(),
                )
                .split(size);

            let copyright = Paragraph::new("Jirust 2023 - all rights reserved")
                .style(Style::default().fg(Color::LightCyan))
                .alignment(Alignment::Center)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .style(Style::default().fg(Color::White))
                        .title("Copyright")
                        .border_type(BorderType::Plain),
                );

            // let menu = menu_titles
            //     .iter()
            //     .map(|t| {
            //         let (first, rest) = t.split_at(1);
            //         Spans::from(vec![
            //             Span::styled(
            //                 first,
            //                 Style::default()
            //                     .fg(Color::Yellow)
            //                     .add_modifier(Modifier::UNDERLINED),
            //             ),
            //             Span::styled(rest, Style::default().fg(Color::White)),
            //         ])
            //     })
            //     .collect();
            //
            // let tabs = Tabs::new(menu)
            //     .select(active_menu_item.into())
            //     .block(Block::default().title("Menu").borders(Borders::ALL))
            //     .style(Style::default().fg(Color::White))
            //     .highlight_style(Style::default().fg(Color::Yellow))
            //     .divider(Span::raw("|"));
            //
            // rect.render_widget(tabs, chunks[0]);
            match active_menu_item {
                MenuItem::Home => rect.render_widget(render_home(), pane[1]),
                MenuItem::Projects => {
                    let project_chunks = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints(
                            [Constraint::Percentage(20), Constraint::Percentage(80)].as_ref(),
                        )
                        .split(pane[1]);
                    let (left, right) = render_projects(&projects_list, &project_list_state);
                    rect.render_stateful_widget(left, project_chunks[0], &mut project_list_state);
                    rect.render_widget(right, project_chunks[1]);
                },
                MenuItem::Issues => {
                    let issue_chunks = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints(
                            [Constraint::Percentage(20), Constraint::Percentage(80)].as_ref(),
                        )
                        .split(pane[1]);
                    let (left, right) = render_issues(&issues_list, &issues_list_state);
                    rect.render_stateful_widget(left, issue_chunks[0], &mut issues_list_state);
                    rect.render_widget(right, issue_chunks[1]);
                }
            }
            rect.render_widget(copyright, pane[2]);
        })?;

        match rx.recv()? {
            Event::Input(event) => match event.code {
                KeyCode::Char('q') => {
                    disable_raw_mode()?;
                    terminal.show_cursor()?;
                    break;
                }
                KeyCode::Char('h') => active_menu_item = MenuItem::Home,
                KeyCode::Char('p') => active_menu_item = MenuItem::Projects,
                // KeyCode::Char('a') => {
                //     add_random_pet_to_db().expect("can add new random pet");
                // }
                // KeyCode::Char('d') => {
                //     remove_pet_at_index(&mut pet_list_state).expect("can remove pet");
                // }
                KeyCode::Enter => {
                    if let Some(selected) = project_list_state.selected() {
                        let project_name = &projects_list[selected];
                        issues_list =
                            jira_issues.get_jira_issues(&project_name.to_string())
                            .await
                            .expect("get jira issues list");
                    }
                    active_menu_item = MenuItem::Issues;
                }
                KeyCode::Down => {
                    if let Some(selected) = project_list_state.selected() {
                        let amount_projects = projects_list.len();
                        if selected >= amount_projects - 1 {
                            project_list_state.select(Some(0));
                        } else {
                            project_list_state.select(Some(selected + 1));
                        }
                    }
                }
                KeyCode::Up => {
                    if let Some(selected) = project_list_state.selected() {
                        let amount_projects = projects_list.len();
                        if selected > 0 {
                            project_list_state.select(Some(selected - 1));
                        } else {
                            project_list_state.select(Some(amount_projects - 1));
                        }
                    }
                }
                _ => {}
            },
            Event::Tick => {}
        }
    }

    Ok(())
}
